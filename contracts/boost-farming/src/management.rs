use crate::*;
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;

#[near_bindgen]
impl Contract {

    #[payable]
    pub fn modify_daily_reward(&mut self, farm_id: FarmId, daily_reward: U128) {
        assert_one_yocto();
        require!(self.is_owner_or_operators(), E002_NOT_ALLOWED);
        require!(self.data().state == RunningState::Running, E004_CONTRACT_PAUSED);
        
        let (seed_id, _) = parse_farm_id(&farm_id);
        let mut seed = self.internal_unwrap_seed(&seed_id);

        let VSeedFarm::Current(seed_farm) = seed.farms.get_mut(&farm_id).expect(E401_FARM_NOT_EXIST);
        seed_farm.terms.daily_reward = daily_reward.0;

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

}