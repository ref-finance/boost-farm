use crate::*;

#[derive(BorshSerialize, BorshDeserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(feature = "test", derive(Clone))]
pub struct Farmer {
    /// A copy of an farmer ID. Saves one storage_read when iterating on farmers.
    pub farmer_id: AccountId,
    pub sponsor_id: AccountId,
    /// Amounts of various reward tokens the farmer claimed.
    pub rewards: HashMap<AccountId, Balance>,
    /// Various seed tokens the farmer staked.
    #[serde(skip)]
    pub seeds: UnorderedMap<SeedId, FarmerSeedOld>,
    #[serde(skip)]
    pub vseeds: UnorderedMap<SeedId, VFarmerSeed>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VFarmer {
    V0(FarmerV0),
    V1(FarmerV1),
    Current(Farmer),
}

impl From<VFarmer> for Farmer {
    fn from(v: VFarmer) -> Self {
        match v {
            VFarmer::V0(c) => c.into(),
            VFarmer::V1(c) => c.into(),
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
            seeds: UnorderedMap::new(StorageKeys::FarmerSeed {
                account_id: farmer_id.clone(),
            }),
            vseeds: UnorderedMap::new(StorageKeys::VFarmerSeed {
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
        } else {
            env::panic_str(format!("Farmer does not have {} reward.", token_id).as_str());
        }
    }

    pub fn get_seed_unwrap(&self, seed_id: &SeedId) -> FarmerSeed {
        if let Some(seed) = self.seeds.get(seed_id) {
            seed.into()
        } else {
            self.vseeds.get(seed_id).unwrap().into()
        }
    }

    pub fn get_seed(&self, seed_id: &SeedId) -> Option<FarmerSeed> {
        if let Some(seed) = self.seeds.get(seed_id) {
            Some(seed.into())
        } else {
            self.vseeds.get(seed_id).map(|v| v.into())
        }
    }

    pub fn remove_seed(&mut self, seed_id: &SeedId) {
        if self.seeds.remove(seed_id).is_none() {
            self.vseeds.remove(seed_id);
        }
    }

    pub fn set_seed(&mut self, seed_id: &SeedId, seed: FarmerSeed) {
        self.seeds.remove(seed_id);
        self.vseeds.insert(seed_id, &seed.into());
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
            .get_seed(&seed.seed_id)
            .unwrap_or_else(|| FarmerSeed {
                free_amount: 0,
                shadow_amount: 0,
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
                VSeedFarm::V0(farm) => {
                    (farm.rps, farm.terms.reward_token.clone(), farm.total_reward)
                }
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

        farmer.set_seed(&seed.seed_id, farmer_seed);
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
