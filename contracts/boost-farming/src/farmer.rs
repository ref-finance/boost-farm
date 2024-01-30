use crate::*;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Default, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, Deserialize))]
#[serde(crate = "near_sdk::serde")]
pub struct FarmerWithdraw {
    #[serde(with = "u128_dec_format")]
    pub amount: Balance,
    #[serde(with = "u64_dec_format")]
    pub apply_timestamp: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(feature = "test", derive(Clone))]
pub struct Farmer {
    /// A copy of an farmer ID. Saves one storage_read when iterating on farmers.
    pub farmer_id: AccountId,
    pub sponsor_id: AccountId,
    /// Amounts of various reward tokens the farmer claimed.
    pub rewards: HashMap<AccountId, Balance>,
    pub withdraws: HashMap<SeedId, FarmerWithdraw>,
    /// Various seed tokens the farmer staked.
    #[serde(skip_serializing)]
    pub seeds: UnorderedMap<SeedId, FarmerSeed>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VFarmer {
    Current(Farmer),
}

impl From<VFarmer> for Farmer {
    fn from(v: VFarmer) -> Self {
        match v {
            VFarmer::Current(c) => c,
        }
    }
}

impl From<Farmer> for VFarmer {
    fn from(c: Farmer) -> Self {
        VFarmer::Current(c)
    }
}

impl Farmer {
    pub fn new(farmer_id: &AccountId, sponsor_id: &AccountId) -> Self {
        Farmer {
            farmer_id: farmer_id.clone(),
            sponsor_id: sponsor_id.clone(),
            rewards: HashMap::new(),
            withdraws: HashMap::new(),
            seeds: UnorderedMap::new(StorageKeys::FarmerSeed {
                account_id: farmer_id.clone(),
            }),
        }
    }
    pub fn add_rewards(&mut self, rewards: &HashMap<AccountId, Balance>) {
        for (reward_token, reward) in rewards {
            self.rewards.insert(
                reward_token.clone(),
                (reward + self.rewards.get(reward_token).unwrap_or(&0_u128)).clone(),
            );
        }
    }

    pub fn sub_reward(&mut self, token_id: &AccountId, amount: Balance) {
        if let Some(prev) = self.rewards.remove(token_id) {
            require!(amount <= prev, E101_INSUFFICIENT_BALANCE);
            let remain = prev - amount;
            if remain > 0 {
                self.rewards.insert(token_id.clone(), remain);
            }
        }
    }

    pub fn add_withdraw_seed(&mut self, seed_id: &SeedId, amount: Balance) {
        if let Some(mut withdraw_seed) = self.withdraws.get_mut(seed_id) {
            withdraw_seed.amount += amount;
            withdraw_seed.apply_timestamp = env::block_timestamp();
        } else {
            self.withdraws.insert(seed_id.clone(), FarmerWithdraw {
                amount,
                apply_timestamp: env::block_timestamp(),
            });
        }
    }

    pub fn sub_withdraw_seed(&mut self, seed_id: &SeedId, amount: Balance, lock_duration: DurationSec) {
        if let Some(prev) = self.withdraws.remove(seed_id) {
            require!(amount <= prev.amount, E101_INSUFFICIENT_BALANCE);
            require!(env::block_timestamp() >= prev.apply_timestamp + to_nano(lock_duration), E305_STILL_IN_LOCK);
            let remain = prev.amount - amount;
            if remain > 0 {
                self.withdraws.insert(seed_id.clone(), FarmerWithdraw {
                    amount: remain,
                    apply_timestamp: prev.apply_timestamp,
                });
            }
        }
    }
}

impl Contract {

    /// return updated FarmerSeed, reward balance per reward token and claimed balance per farm
    pub fn internal_calc_farmer_claim(
        &self,
        farmer: &Farmer,
        seed: &Seed,
    ) -> (
        FarmerSeed,
        HashMap<AccountId, Balance>,
        HashMap<FarmId, Balance>,
    ) {
        let mut rewards = HashMap::new();
        let mut claimed = HashMap::new();

        let mut farmer_seed: FarmerSeed = farmer
            .seeds
            .get(&seed.seed_id)
            .map(|v| v.into())
            .unwrap_or_else(|| FarmerSeed {
                free_amount: 0,
                locked_amount: 0,
                x_locked_amount: 0,
                unlock_timestamp: 0,
                duration_sec: 0,
                boost_ratios: self.gen_booster_ratios(&seed.seed_id, farmer),
                user_rps: HashMap::new(),
            });

        let farmer_seed_power = farmer_seed.get_seed_power();

        let mut new_user_rps = HashMap::new();
        for (farm_id, vfarm) in &seed.farms {
            let (seed_farm_rps, seed_farm_terms_reward_token, seed_farm_total_reward) = match vfarm {
                VSeedFarm::Current(farm) => {
                    (farm.rps, farm.terms.reward_token.clone(), farm.total_reward)
                }
            };
            let farmer_rps = farmer_seed.user_rps.get(farm_id).unwrap_or(&BigDecimal::zero()).clone();
            let diff = seed_farm_rps - farmer_rps;
            let reward_amount = diff.round_down_mul_u128(farmer_seed_power);
            if reward_amount > 0 {
                rewards.insert(
                    seed_farm_terms_reward_token.clone(),
                    reward_amount
                        + rewards
                            .get(&seed_farm_terms_reward_token)
                            .unwrap_or(&0_u128),
                );
                claimed.insert(farm_id.clone(), reward_amount);
            }

            // bypass non-reward
            if seed_farm_total_reward > 0 {
                new_user_rps.insert(farm_id.clone(), seed_farm_rps);
            }
        }
        farmer_seed.user_rps = new_user_rps;

        (farmer_seed, rewards, claimed)
    }

    pub fn internal_do_farmer_claim(&self, farmer: &mut Farmer, seed: &mut Seed) {
        let (mut farmer_seed, rewards, claimed) = self.internal_calc_farmer_claim(&farmer, &seed);
        farmer.add_rewards(&rewards);
        
        // sync booster info
        let prev = farmer_seed.get_seed_power();
        farmer_seed.boost_ratios = self.gen_booster_ratios(&seed.seed_id, farmer);
        seed.total_seed_power = seed.total_seed_power + farmer_seed.get_seed_power() - prev;

        farmer.seeds.insert(&seed.seed_id, &farmer_seed);
        seed.update_claimed(&claimed);

    }

    pub fn internal_get_farmer(&self, farmer_id: &AccountId) -> Option<Farmer> {
        self.data().farmers.get(farmer_id).map(|o| o.into())
    }

    pub fn internal_unwrap_farmer(&self, farmer_id: &AccountId) -> Farmer {
        self.internal_get_farmer(farmer_id)
            .expect(E100_ACC_NOT_REGISTERED)
    }

    pub fn internal_set_farmer(&mut self, farmer_id: &AccountId, farmer: Farmer) {
        self.data_mut().farmers.insert(farmer_id, &farmer.into());
    }
}
