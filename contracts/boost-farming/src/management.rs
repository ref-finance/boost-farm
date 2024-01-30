use crate::*;
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;

#[near_bindgen]
impl Contract {

    #[payable]
    pub fn modify_delay_withdraw_sec(&mut self, delay_withdraw_sec: DurationSec) {
        assert_one_yocto();
        require!(self.is_owner_or_operators(), E002_NOT_ALLOWED);
        require!(self.data().state == RunningState::Running, E004_CONTRACT_PAUSED);
        let mut config =  self.data().config.get().unwrap();
        config.delay_withdraw_sec = delay_withdraw_sec;        
        config.assert_valid();
        self.data_mut().config.set(&config);
    }

    #[payable]
    pub fn modify_daily_reward(&mut self, farm_id: FarmId, daily_reward: U128) {
        assert_one_yocto();
        require!(self.is_owner_or_operators(), E002_NOT_ALLOWED);
        require!(self.data().state == RunningState::Running, E004_CONTRACT_PAUSED);
        
        let (seed_id, _) = parse_farm_id(&farm_id);
        let mut seed = self.internal_unwrap_seed(&seed_id);

        let vfarm = seed.farms.get_mut(&farm_id).expect(E401_FARM_NOT_EXIST);
        match vfarm {
            VSeedFarm::Current(farm) => {
                farm.terms.daily_reward = daily_reward.0;
            }
        }

        self.internal_set_seed(&seed_id, seed);
    }

    #[payable]
    pub fn modify_locking_policy(&mut self, max_duration: DurationSec, max_ratio: u32) {
        assert_one_yocto();
        require!(self.is_owner_or_operators(), E002_NOT_ALLOWED);
        require!(self.data().state == RunningState::Running, E004_CONTRACT_PAUSED);
        
        let mut config =  self.data().config.get().unwrap();
        // config.minimum_staking_duration_sec = min_duration;
        config.maximum_locking_duration_sec = max_duration;
        // config.min_booster_multiplier = min_ratio;
        config.max_locking_multiplier = max_ratio;
        
        config.assert_valid();
        self.data_mut().config.set(&config);
    }

    #[payable]
    pub fn modify_max_farm_num_per_seed(&mut self, max_num: u32) {
        assert_one_yocto();
        require!(self.is_owner_or_operators(), E002_NOT_ALLOWED);
        require!(self.data().state == RunningState::Running, E004_CONTRACT_PAUSED);
        
        let mut config =  self.data().config.get().unwrap();
        config.max_num_farms_per_seed = max_num;
        self.data_mut().config.set(&config);
    }

    #[payable]
    pub fn modify_default_slash_rate(&mut self, slash_rate: u32) {
        assert_one_yocto();
        require!(self.is_owner_or_operators(), E002_NOT_ALLOWED);
        require!(self.data().state == RunningState::Running, E004_CONTRACT_PAUSED);
        require!(BP_DENOM > slash_rate as u128, E205_INVALID_SLASH_RATE);
        
        let mut config =  self.data().config.get().unwrap();
        config.seed_slash_rate = slash_rate;
        self.data_mut().config.set(&config);
    }

    #[payable]
    pub fn modify_seed_min_deposit(&mut self, seed_id: String, min_deposit: U128) {
        assert_one_yocto();
        require!(self.is_owner_or_operators(), E002_NOT_ALLOWED);
        require!(self.data().state == RunningState::Running, E004_CONTRACT_PAUSED);

        let mut seed = self.internal_unwrap_seed(&seed_id);
        seed.min_deposit = min_deposit.into();
        self.internal_set_seed(&seed_id, seed);
    }

    #[payable]
    pub fn modify_seed_min_locking_duration(&mut self, seed_id: String, min_locking_duration_sec: DurationSec) {
        assert_one_yocto();
        require!(self.is_owner_or_operators(), E002_NOT_ALLOWED);
        require!(self.data().state == RunningState::Running, E004_CONTRACT_PAUSED);

        let config = self.internal_config();
        require!(min_locking_duration_sec<=config.maximum_locking_duration_sec, E201_INVALID_DURATION);
        let mut seed = self.internal_unwrap_seed(&seed_id);
        seed.min_locking_duration_sec = min_locking_duration_sec;
        self.internal_set_seed(&seed_id, seed);
    }

    #[payable]
    pub fn modify_seed_slash_rate(&mut self, seed_id: String, slash_rate: u32) {
        assert_one_yocto();
        require!(self.is_owner_or_operators(), E002_NOT_ALLOWED);
        require!(self.data().state == RunningState::Running, E004_CONTRACT_PAUSED);

        let mut seed = self.internal_unwrap_seed(&seed_id);
        seed.slash_rate = slash_rate;
        self.internal_set_seed(&seed_id, seed);
    }
    
    /// Owner retrieve those slashed seed
    #[payable]
    pub fn withdraw_seed_slashed(&mut self, seed_id: SeedId) -> Promise {
        assert_one_yocto();
        require!(self.is_owner_or_operators(), E002_NOT_ALLOWED);
        require!(self.data().state == RunningState::Running, E004_CONTRACT_PAUSED);

        // update inner state
        let amount = self.data_mut().seeds_slashed.remove(&seed_id).unwrap_or(0_u128);
        require!(amount > 0, E101_INSUFFICIENT_BALANCE);

        let (token, token_id) = parse_seed_id(&seed_id);

        if let Some(token_id) = token_id {
            ext_multi_fungible_token::mft_transfer(
                wrap_mft_token_id(&token_id),
                self.data().owner_id.clone(),
                amount.into(),
                None,
                token,
                1, // one yocto near
                GAS_FOR_SEED_TRANSFER,
            )
            .then(ext_self::callback_withdraw_seed_slashed(
                seed_id.clone(),
                amount.into(),
                env::current_account_id(),
                0,
                GAS_FOR_RESOLVE_SEED_TRANSFER,
            ))
        } else {
            ext_fungible_token::ft_transfer(
                self.data().owner_id.clone(),
                amount.into(),
                None,
                token,
                1, // one yocto near
                GAS_FOR_SEED_TRANSFER,
            )
            .then(ext_self::callback_withdraw_seed_slashed(
                seed_id.clone(),
                amount.into(),
                env::current_account_id(),
                0,
                GAS_FOR_RESOLVE_SEED_TRANSFER,
            ))
        }
    }

    /// owner help to return those who lost seed when withdraw,
    /// It's owner's responsibility to verify amount and seed id before calling
    #[payable]
    pub fn return_seed_lostfound(&mut self, farmer_id: AccountId, seed_id: SeedId, amount: U128) -> Promise {
        assert_one_yocto();
        self.assert_owner();
        require!(self.data().state == RunningState::Running, E004_CONTRACT_PAUSED);

        self.internal_unwrap_farmer(&farmer_id);

        // update inner state
        let max_amount = self.data().seeds_lostfound.get(&seed_id).unwrap_or(0_u128);
        require!(amount.0 <= max_amount, E101_INSUFFICIENT_BALANCE);
        self.data_mut().seeds_lostfound.insert(&seed_id, &(max_amount - amount.0));

        let (token, token_id) = parse_seed_id(&seed_id);

        if let Some(token_id) = token_id {
            ext_multi_fungible_token::mft_transfer(
                wrap_mft_token_id(&token_id),
                farmer_id.clone(),
                amount.into(),
                None,
                token,
                1, // one yocto near
                GAS_FOR_SEED_TRANSFER,
            )
            .then(ext_self::callback_withdraw_seed_lostfound(
                seed_id.clone(),
                farmer_id.clone(),
                amount.into(),
                env::current_account_id(),
                0,
                GAS_FOR_RESOLVE_SEED_TRANSFER,
            ))
        } else {
            ext_fungible_token::ft_transfer(
                farmer_id.clone(),
                amount.into(),
                None,
                token,
                1, // one yocto near
                GAS_FOR_SEED_TRANSFER,
            )
            .then(ext_self::callback_withdraw_seed_lostfound(
                seed_id.clone(),
                farmer_id.clone(),
                amount.into(),
                env::current_account_id(),
                0,
                GAS_FOR_RESOLVE_SEED_TRANSFER,
            ))
        }
    }

    #[payable]
    pub fn withdraw_from_beneficiary_account(&mut self, farm_id: FarmId) -> Promise {
        assert_one_yocto();
        require!(self.is_owner_or_operators(), E002_NOT_ALLOWED);
        require!(self.data().state == RunningState::Running, E004_CONTRACT_PAUSED);
        
        let (seed_id, _) = parse_farm_id(&farm_id);
        let mut seed = self.internal_unwrap_seed(&seed_id);

        let v_farm = seed.farms.remove(&farm_id).expect(E401_FARM_NOT_EXIST);
        let mut seed_farm = match v_farm {
            VSeedFarm::Current(farm) => {
                farm
            }
        };

        let amount = seed_farm.amount_of_beneficiary - seed_farm.amount_of_withdrew_beneficiary;
        require!(amount > 0, E101_INSUFFICIENT_BALANCE);
        let reward_token = seed_farm.terms.reward_token.clone();
        seed_farm.amount_of_withdrew_beneficiary = seed_farm.amount_of_beneficiary;
        seed.farms.insert(farm_id.clone(), seed_farm.into());
        self.internal_set_seed(&seed_id, seed);

        ext_fungible_token::ft_transfer(
            self.data().owner_id.clone(),
            amount.into(),
            None,
            reward_token,
            1, // one yocto near
            GAS_FOR_REWARD_TRANSFER,
        )
        .then(ext_self::callback_withdraw_beneficiary(
            farm_id.clone(),
            amount.into(),
            env::current_account_id(),
            0,
            GAS_FOR_RESOLVE_REWARD_TRANSFER,
        ))
    }

    #[payable]
    pub fn withdraw_from_outdated_beneficiary_account(&mut self, farm_id: FarmId) -> Promise {
        assert_one_yocto();
        require!(self.is_owner_or_operators(), E002_NOT_ALLOWED);
        require!(self.data().state == RunningState::Running, E004_CONTRACT_PAUSED);
        
        let v_farm = self.data_mut().outdated_farms.remove(&farm_id).expect(E401_FARM_NOT_EXIST);
        let mut seed_farm = match v_farm {
            VSeedFarm::Current(farm) => {
                farm
            }
        };

        let amount = seed_farm.amount_of_beneficiary - seed_farm.amount_of_withdrew_beneficiary;
        require!(amount > 0, E101_INSUFFICIENT_BALANCE);
        let reward_token = seed_farm.terms.reward_token.clone();
        seed_farm.amount_of_withdrew_beneficiary = seed_farm.amount_of_beneficiary;
        self.data_mut().outdated_farms.insert(&farm_id, &seed_farm.into());

        ext_fungible_token::ft_transfer(
            self.data().owner_id.clone(),
            amount.into(),
            None,
            reward_token,
            1, // one yocto near
            GAS_FOR_REWARD_TRANSFER,
        )
        .then(ext_self::callback_withdraw_outdated_beneficiary(
            farm_id.clone(),
            amount.into(),
            env::current_account_id(),
            0,
            GAS_FOR_RESOLVE_REWARD_TRANSFER,
        ))
    }

    #[payable]
    pub fn withdraw_from_undistributed_reward(&mut self, farm_id: FarmId, amount: U128) -> Promise {
        assert_one_yocto();
        require!(self.is_owner_or_operators(), E002_NOT_ALLOWED);
        require!(self.data().state == RunningState::Running, E004_CONTRACT_PAUSED);

        let mut amount: Balance = amount.into();
        
        let (seed_id, _) = parse_farm_id(&farm_id);
        let mut seed = self.internal_unwrap_seed(&seed_id);

        let v_farm = seed.farms.remove(&farm_id).expect(E401_FARM_NOT_EXIST);
        let mut seed_farm = match v_farm {
            VSeedFarm::Current(farm) => {
                farm
            }
        };
        
        let amount_availabe = seed_farm.total_reward - seed_farm.distributed_reward;
        if amount == 0 {
            amount = amount_availabe;
        }
        require!(amount <= amount_availabe && amount_availabe > 0, E101_INSUFFICIENT_BALANCE);
        let reward_token = seed_farm.terms.reward_token.clone();
        seed_farm.total_reward -= amount;
        seed.farms.insert(farm_id.clone(), seed_farm.into());
        self.internal_set_seed(&seed_id, seed);

        ext_fungible_token::ft_transfer(
            self.data().owner_id.clone(),
            amount.into(),
            None,
            reward_token,
            1, // one yocto near
            GAS_FOR_REWARD_TRANSFER,
        )
        .then(ext_self::callback_withdraw_undistributed(
            farm_id.clone(),
            amount.into(),
            env::current_account_id(),
            0,
            GAS_FOR_RESOLVE_REWARD_TRANSFER,
        ))
    }

    #[private]
    pub fn callback_withdraw_seed_lostfound(&mut self, seed_id: SeedId, sender_id: AccountId, amount: U128) {
        require!(
            env::promise_results_count() == 1,
            E001_PROMISE_RESULT_COUNT_INVALID
        );
        let amount: Balance = amount.into();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => {
                // all seed amount go to lostfound
                let seed_amount = self.data().seeds_lostfound.get(&seed_id).unwrap_or(0);
                self.data_mut()
                    .seeds_lostfound
                    .insert(&seed_id, &(seed_amount + amount));

                Event::SeedWithdrawLostfound {
                    farmer_id: &sender_id,
                    seed_id: &seed_id,
                    withdraw_amount: &U128(amount),
                    success: false,
                }
                .emit();
            },
            PromiseResult::Successful(_) => {
                Event::SeedWithdrawLostfound {
                    farmer_id: &sender_id,
                    seed_id: &seed_id,
                    withdraw_amount: &U128(amount),
                    success: true,
                }
                .emit();
            }
        }
    }

    /// if withdraw seed slashed encounter async error, it would go back to seeds_slashed
    #[private]
    pub fn callback_withdraw_seed_slashed(&mut self, seed_id: SeedId, amount: U128) {
        require!(
            env::promise_results_count() == 1,
            E001_PROMISE_RESULT_COUNT_INVALID
        );
        let amount: Balance = amount.into();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => {
                // all seed amount go back to seed slashed
                let seed_amount = self.data().seeds_slashed.get(&seed_id).unwrap_or(0);
                self.data_mut().seeds_slashed.insert(&seed_id, &(seed_amount + amount));
                Event::SeedWithdrawSlashed {
                    owner_id: &self.data().owner_id,
                    seed_id: &seed_id,
                    withdraw_amount: &U128(amount),
                    success: false,
                }
                .emit();
            },
            PromiseResult::Successful(_) => {
                Event::SeedWithdrawSlashed {
                    owner_id: &self.data().owner_id,
                    seed_id: &seed_id,
                    withdraw_amount: &U128(amount),
                    success: true,
                }
                .emit();
            }
        }
    }

    #[private]
    pub fn callback_withdraw_beneficiary(&mut self, farm_id: FarmId, amount: U128) {
        require!(
            env::promise_results_count() == 1,
            E001_PROMISE_RESULT_COUNT_INVALID
        );
        let amount: Balance = amount.into();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => {
                // add amount back to amount_of_beneficiary,
                // without touching any seed distribution logic to save gas consumption
                let (seed_id, _) = parse_farm_id(&farm_id);
                let mut seed: Seed = self.data().seeds.get(&seed_id).map(|v| v.into()).expect(E301_SEED_NOT_EXIST);
                let VSeedFarm::Current(farm) = seed.farms.get_mut(&farm_id).expect(E401_FARM_NOT_EXIST);
                farm.amount_of_withdrew_beneficiary -= amount;
                self.data_mut().seeds.insert(&seed_id, &seed.into());

                Event::RewardWithdrawBeneficiary {
                    owner_id: &self.data().owner_id,
                    farm_id: &farm_id,
                    withdraw_amount: &U128(amount),
                    success: false,
                }
                .emit();
            },
            PromiseResult::Successful(_) => {
                Event::RewardWithdrawBeneficiary {
                    owner_id: &self.data().owner_id,
                    farm_id: &farm_id,
                    withdraw_amount: &U128(amount),
                    success: true,
                }
                .emit();
            }
        }
    }

    #[private]
    pub fn callback_withdraw_outdated_beneficiary(&mut self, farm_id: FarmId, amount: U128) {
        require!(
            env::promise_results_count() == 1,
            E001_PROMISE_RESULT_COUNT_INVALID
        );
        let amount: Balance = amount.into();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => {
                let VSeedFarm::Current(mut farm) = self.data_mut().outdated_farms.remove(&farm_id).expect(E401_FARM_NOT_EXIST);
                farm.amount_of_withdrew_beneficiary -= amount;
                self.data_mut().outdated_farms.insert(&farm_id, &farm.into());
                
                Event::RewardWithdrawOutdatedBeneficiary {
                    owner_id: &self.data().owner_id,
                    farm_id: &farm_id,
                    withdraw_amount: &U128(amount),
                    success: false,
                }
                .emit();
            },
            PromiseResult::Successful(_) => {
                Event::RewardWithdrawOutdatedBeneficiary {
                    owner_id: &self.data().owner_id,
                    farm_id: &farm_id,
                    withdraw_amount: &U128(amount),
                    success: true,
                }
                .emit();
            }
        }
    }

    #[private]
    pub fn callback_withdraw_undistributed(&mut self, farm_id: FarmId, amount: U128) {
        require!(
            env::promise_results_count() == 1,
            E001_PROMISE_RESULT_COUNT_INVALID
        );
        let amount: Balance = amount.into();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => {
                // add amount back to total_reward,
                // without touching any seed distribution logic to save gas consumption
                let (seed_id, _) = parse_farm_id(&farm_id);
                let mut seed: Seed = self.data().seeds.get(&seed_id).map(|v| v.into()).expect(E301_SEED_NOT_EXIST);
                let VSeedFarm::Current(farm) = seed.farms.get_mut(&farm_id).expect(E401_FARM_NOT_EXIST);
                farm.total_reward += amount;

                self.data_mut().seeds.insert(&seed_id, &seed.into());

                Event::RewardWithdrawUndistributed {
                    owner_id: &self.data().owner_id,
                    farm_id: &farm_id,
                    withdraw_amount: &U128(amount),
                    success: false,
                }
                .emit();
            },
            PromiseResult::Successful(_) => {
                Event::RewardWithdrawUndistributed {
                    owner_id: &self.data().owner_id,
                    farm_id: &farm_id,
                    withdraw_amount: &U128(amount),
                    success: true,
                }
                .emit();
            }
        }
    }
}