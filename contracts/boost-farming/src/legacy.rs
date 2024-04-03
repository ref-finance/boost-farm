use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ContractDataV0100 {
    pub owner_id: AccountId,
    pub operators: UnorderedSet<AccountId>,
    pub config: LazyOption<ConfigV0>,
    pub seeds: UnorderedMap<SeedId, VSeed>,
    pub farmers: LookupMap<AccountId, VFarmer>,
    pub outdated_farms: UnorderedMap<FarmId, VSeedFarm>,
    // all slashed seed would recorded in here
    pub seeds_slashed: UnorderedMap<SeedId, Balance>,
    // if unstake seed encounter error, the seed would go to here
    pub seeds_lostfound: UnorderedMap<SeedId, Balance>,

    // for statistic
    farmer_count: u64,
    farm_count: u64,
}

impl From<ContractDataV0100> for ContractData {
    fn from(a: ContractDataV0100) -> Self {
        let ContractDataV0100 {
            owner_id,
            operators,
            config,
            seeds,
            farmers,
            outdated_farms,
            seeds_slashed,
            seeds_lostfound,
            farmer_count,
            farm_count,
        } = a;
        let config_v0 = config.get().unwrap();
        Self {
            owner_id: owner_id.clone(),
            next_owner_id: None,
            next_owner_accept_deadline: None,
            ref_exchange_id: owner_id,
            operators,
            config: LazyOption::new(StorageKeys::Config, Some(&config_v0.into())),
            seeds,
            farmers,
            outdated_farms,
            seeds_slashed,
            seeds_lostfound,
            farmer_count,
            farm_count,
            state: RunningState::Running,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct FarmerV0 {
    /// A copy of an farmer ID. Saves one storage_read when iterating on farmers.
    pub farmer_id: AccountId,
    /// Amounts of various reward tokens the farmer claimed.
    pub rewards: HashMap<AccountId, Balance>,
    /// Various seed tokens the farmer staked.
    pub seeds: UnorderedMap<SeedId, FarmerSeedOld>,
}

impl From<FarmerV0> for Farmer {
    fn from(a: FarmerV0) -> Self {
        let FarmerV0 {
            farmer_id,
            rewards,
            seeds,
        } = a;
        Self {
            farmer_id: farmer_id.clone(),
            sponsor_id: farmer_id.clone(),
            rewards,
            seeds,
            vseeds: UnorderedMap::new(StorageKeys::VFarmerSeed {
                account_id: farmer_id.clone(),
            }),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ContractDataV0101 {
    pub owner_id: AccountId,
    pub state: RunningState,
    pub operators: UnorderedSet<AccountId>,
    pub config: LazyOption<ConfigV0>,
    pub seeds: UnorderedMap<SeedId, VSeed>,
    pub farmers: LookupMap<AccountId, VFarmer>,
    pub outdated_farms: UnorderedMap<FarmId, VSeedFarm>,
    // all slashed seed would recorded in here
    pub seeds_slashed: UnorderedMap<SeedId, Balance>,
    // if unstake seed encounter error, the seed would go to here
    pub seeds_lostfound: UnorderedMap<SeedId, Balance>,

    // for statistic
    farmer_count: u64,
    farm_count: u64,
}

impl From<ContractDataV0101> for ContractData {
    fn from(a: ContractDataV0101) -> Self {
        let ContractDataV0101 {
            owner_id,
            state,
            operators,
            config,
            seeds,
            farmers,
            outdated_farms,
            seeds_slashed,
            seeds_lostfound,
            farmer_count,
            farm_count,
        } = a;
        let config_v0 = config.get().unwrap();
        Self {
            owner_id: owner_id.clone(),
            next_owner_id: None,
            next_owner_accept_deadline: None,
            ref_exchange_id: owner_id,
            state,
            operators,
            config: LazyOption::new(StorageKeys::Config, Some(&config_v0.into())),
            seeds,
            farmers,
            outdated_farms,
            seeds_slashed,
            seeds_lostfound,
            farmer_count,
            farm_count,
            
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, Deserialize))]
#[serde(crate = "near_sdk::serde")]
pub struct SeedFarmV0 {
    pub farm_id: FarmId,

    pub terms: FarmTerms,

    /// total reward send into this farm by far,
    /// every time reward deposited in, add to this field
    #[serde(with = "u128_dec_format")]
    pub total_reward: Balance,

    #[serde(with = "u64_dec_format")]
    pub distributed_at: Timestamp,

    /// The amount of rewards has been distributed.
    /// remaining_reward = total_reward - distributed_reward
    #[serde(with = "u128_dec_format")]
    pub distributed_reward: Balance,

    /// reward token has been claimed by farmer
    #[serde(with = "u128_dec_format")]
    pub claimed_reward: Balance,

    /// when there is no seed token staked, reward goes to beneficiary
    #[serde(with = "u128_dec_format")]
    pub amount_of_beneficiary: Balance,

    #[serde(skip)]
    pub rps: BigDecimal,

    #[borsh_skip]
    pub status: Option<FarmStatus>,
}

impl SeedFarmV0 {
    pub fn has_ended(&self) -> bool {
        match self.status.as_ref().unwrap() {
            FarmStatus::Ended => true,
            _ => false,
        }
    }

    pub fn internal_update_status(&mut self, block_ts: u64) {
        if self.terms.start_at == 0 || to_nano(self.terms.start_at) >= block_ts {
            self.status = Some(FarmStatus::Created);
        } else if self.total_reward == 0 && to_nano(self.terms.start_at) < block_ts {
            self.status = Some(FarmStatus::Pending);
        } else if self.total_reward > 0 && self.distributed_reward >= self.total_reward {
            self.status = Some(FarmStatus::Ended);
        } else {
            self.status = Some(FarmStatus::Running);
        }
    }

    pub fn update(&mut self, seed_power: Balance) {
        let block_ts = env::block_timestamp();

        self.internal_update_status(block_ts);

        if block_ts <= self.distributed_at {
            // already updated, skip
            return;
        }

        match self.status.as_ref().unwrap() {
            FarmStatus::Ended => {
                self.distributed_at = block_ts;
            },
            FarmStatus::Running => {
                let reward = std::cmp::min(
                    self.total_reward - self.distributed_reward,
                    u128_ratio(
                        self.terms.daily_reward,
                        u128::from(block_ts - self.distributed_at),
                        u128::from(NANOS_PER_DAY),
                    ),
                );
                self.distributed_reward += reward;
                if seed_power > 0 {
                    self.rps = self.rps + BigDecimal::from(reward).div_u128(seed_power);
                } else {
                    self.amount_of_beneficiary += reward;
                }
                self.distributed_at = block_ts;
                self.internal_update_status(block_ts);
            },
            _ => {},
        }
    }

    pub fn add_reward(&mut self, reward_token: &AccountId, amount: Balance) -> (Balance, u32) {
        require!(self.terms.reward_token == reward_token.clone(), E404_UNMATCHED_REWARD_TOKEN);
        if self.terms.start_at == 0 {
            self.terms.start_at = nano_to_sec(env::block_timestamp());
            self.distributed_at = env::block_timestamp();
        }
        self.total_reward += amount;
        (self.total_reward, self.terms.start_at)
    }

    pub fn finalize(&mut self) {
        require!(self.has_ended(), E405_FARM_NOT_ENDED);
        // remaining unclaimed rewards belongs to beneficiary
        self.amount_of_beneficiary = 
            self.distributed_reward - self.claimed_reward;
    }

}

impl From<SeedFarmV0> for SeedFarm {
    fn from(a: SeedFarmV0) -> Self {
        let SeedFarmV0 {
            farm_id,
            terms,
            total_reward,
            distributed_at,
            distributed_reward,
            claimed_reward,
            amount_of_beneficiary,
            rps,
            status
        } = a;
        Self {
            farm_id,
            terms,
            total_reward,
            distributed_at,
            distributed_reward,
            claimed_reward,
            amount_of_beneficiary,
            amount_of_withdrew_beneficiary: 0,
            rps,
            status
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ContractDataV0102 {
    pub owner_id: AccountId,
    pub next_owner_id: Option<AccountId>,
    pub next_owner_accept_deadline: Option<u64>,
    pub state: RunningState,
    pub operators: UnorderedSet<AccountId>,
    pub config: LazyOption<ConfigV0>,
    pub seeds: UnorderedMap<SeedId, VSeed>,
    pub farmers: LookupMap<AccountId, VFarmer>,
    pub outdated_farms: UnorderedMap<FarmId, VSeedFarm>,
    // all slashed seed would recorded in here
    pub seeds_slashed: UnorderedMap<SeedId, Balance>,
    // if unstake seed encounter error, the seed would go to here
    pub seeds_lostfound: UnorderedMap<SeedId, Balance>,

    // for statistic
    farmer_count: u64,
    farm_count: u64,
}

impl From<ContractDataV0102> for ContractData {
    fn from(a: ContractDataV0102) -> Self {
        let ContractDataV0102 {
            owner_id,
            next_owner_id,
            next_owner_accept_deadline,
            state,
            operators,
            config,
            seeds,
            farmers,
            outdated_farms,
            seeds_slashed,
            seeds_lostfound,
            farmer_count,
            farm_count,
        } = a;
        let config_v0 = config.get().unwrap();
        Self {
            owner_id: owner_id.clone(),
            next_owner_id,
            next_owner_accept_deadline,
            ref_exchange_id: owner_id,
            state,
            operators,
            config: LazyOption::new(StorageKeys::Config, Some(&config_v0.into())),
            seeds,
            farmers,
            outdated_farms,
            seeds_slashed,
            seeds_lostfound,
            farmer_count,
            farm_count,
            
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct FarmerSeedOld {
    pub free_amount: Balance,
    /// The amount of locked token.
    pub locked_amount: Balance,
    /// The amount of power for those locked amount.
    pub x_locked_amount: Balance,
    /// When the locking token can be unlocked without slash in nanoseconds.
    pub unlock_timestamp: u64,
    /// The duration of current locking in seconds.
    pub duration_sec: u32,
    /// <booster_id, booster-ratio>
    pub boost_ratios: HashMap<SeedId, f64>,
    pub user_rps: HashMap<FarmId, BigDecimal>,
}

impl From<FarmerSeedOld> for FarmerSeed {
    fn from(a: FarmerSeedOld) -> Self {
        let FarmerSeedOld {
            free_amount,
            locked_amount,
            x_locked_amount,
            unlock_timestamp,
            duration_sec,
            boost_ratios,
            user_rps
        } = a;
        Self {
            free_amount,
            shadow_amount: 0,
            locked_amount,
            x_locked_amount,
            unlock_timestamp,
            duration_sec,
            boost_ratios,
            user_rps
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct FarmerV1 {
    /// A copy of an farmer ID. Saves one storage_read when iterating on farmers.
    pub farmer_id: AccountId,
    pub sponsor_id: AccountId,
    /// Amounts of various reward tokens the farmer claimed.
    pub rewards: HashMap<AccountId, Balance>,
    /// Various seed tokens the farmer staked.
    pub seeds: UnorderedMap<SeedId, FarmerSeedOld>,
}

impl From<FarmerV1> for Farmer {
    fn from(a: FarmerV1) -> Self {
        let FarmerV1 {
            farmer_id,
            sponsor_id,
            rewards,
            seeds
        } = a;

        Self {
            farmer_id: farmer_id.clone(),
            sponsor_id,
            rewards,
            seeds,
            vseeds: UnorderedMap::new(StorageKeys::VFarmerSeed {
                account_id: farmer_id.clone(),
            }),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ContractDataV0103 {
    pub owner_id: AccountId,
    pub next_owner_id: Option<AccountId>,
    pub next_owner_accept_deadline: Option<u64>,
    pub ref_exchange_id: AccountId,
    pub state: RunningState,
    pub operators: UnorderedSet<AccountId>,
    pub config: LazyOption<ConfigV0>,
    pub seeds: UnorderedMap<SeedId, VSeed>,
    pub farmers: LookupMap<AccountId, VFarmer>,
    pub outdated_farms: UnorderedMap<FarmId, VSeedFarm>,
    // all slashed seed would recorded in here
    pub seeds_slashed: UnorderedMap<SeedId, Balance>,
    // if unstake seed encounter error, the seed would go to here
    pub seeds_lostfound: UnorderedMap<SeedId, Balance>,

    // for statistic
    farmer_count: u64,
    farm_count: u64,
}

impl From<ContractDataV0103> for ContractData {
    fn from(a: ContractDataV0103) -> Self {
        let ContractDataV0103 {
            owner_id,
            next_owner_id,
            next_owner_accept_deadline,
            ref_exchange_id,
            state,
            operators,
            config,
            seeds,
            farmers,
            outdated_farms,
            seeds_slashed,
            seeds_lostfound,
            farmer_count,
            farm_count,
        } = a;
        let config_v0 = config.get().unwrap();
        Self {
            owner_id,
            next_owner_id,
            next_owner_accept_deadline,
            ref_exchange_id,
            state,
            operators,
            config: LazyOption::new(StorageKeys::Config, Some(&config_v0.into())),
            seeds,
            farmers,
            outdated_farms,
            seeds_slashed,
            seeds_lostfound,
            farmer_count,
            farm_count,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct BoosterInfoV0 {
    pub booster_decimal: u32,
    /// <affected_seed_id, log_base>
    pub affected_seeds: HashMap<SeedId, u32>,
}

impl From<BoosterInfoV0> for BoosterInfo {
    fn from(a: BoosterInfoV0) -> Self {
        let BoosterInfoV0 {
            booster_decimal,
            affected_seeds, 
        } = a;
        
        Self {
            booster_decimal,
            affected_seeds,
            boost_suppress_factor: 1
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(feature = "test", derive(Deserialize, Clone))]
pub struct ConfigV0 {
    pub seed_slash_rate: u32,

    /// Key is boosterID, support multiple booster
    pub booster_seeds: HashMap<SeedId, BoosterInfoV0>,

    pub max_num_farms_per_booster: u32,

    pub max_num_farms_per_seed: u32,

    /// The maximum duration to stake booster token in seconds.
    pub maximum_locking_duration_sec: DurationSec,

    /// The rate of x for the amount of seed given for the maximum locking duration.
    /// Assuming the 100% multiplier at the 0 duration. Should be no less than 100%.
    /// E.g. 20000 means 200% multiplier (or 2X).
    pub max_locking_multiplier: u32,
}

impl From<ConfigV0> for Config {
    fn from(a: ConfigV0) -> Self {
        let ConfigV0 {
            seed_slash_rate,
            booster_seeds, 
            max_num_farms_per_booster,
            max_num_farms_per_seed,
            maximum_locking_duration_sec,
            max_locking_multiplier,
        } = a;
        
        Self {
            seed_slash_rate,
            booster_seeds: booster_seeds.into_iter().map(|(k, v)| (k, v.into())).collect(), 
            max_num_farms_per_booster,
            max_num_farms_per_seed,
            maximum_locking_duration_sec,
            max_locking_multiplier,
        }
    }
}