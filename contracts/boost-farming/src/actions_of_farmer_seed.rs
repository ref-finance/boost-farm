use crate::*;
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;

#[near_bindgen]
impl Contract {

    /// convert free seed to locking mode
    #[payable]
    pub fn lock_free_seed(&mut self, seed_id: SeedId, duration_sec: u32, amount: Option<U128>) {
        assert_one_yocto();
        require!(self.data().state == RunningState::Running, E004_CONTRACT_PAUSED);

        let farmer_id = env::predecessor_account_id();

        let mut farmer = self.internal_unwrap_farmer(&farmer_id);
        let mut seed = self.internal_unwrap_seed(&seed_id);

        require!(seed.min_locking_duration_sec > 0, E300_FORBID_LOCKING);
        require!(duration_sec >= seed.min_locking_duration_sec, E201_INVALID_DURATION);
        let config = self.internal_config();
        require!(duration_sec <= config.maximum_locking_duration_sec, E201_INVALID_DURATION);

        self.internal_do_farmer_claim(&mut farmer, &mut seed);

        let mut farmer_seed = farmer.seeds.get(&seed_id).unwrap();
        let amount = if let Some(request) = amount {
            request.0
        } else {
            farmer_seed.free_amount
        };

        let increased_seed_power =
            farmer_seed.free_to_lock(amount, duration_sec, &config);
        farmer.seeds.insert(&seed_id, &farmer_seed);

        seed.total_seed_power += increased_seed_power;

        self.update_impacted_seeds(&mut farmer, &seed_id);

        self.internal_set_farmer(&farmer_id, farmer);
        self.internal_set_seed(&seed_id, seed);

        Event::SeedFreeToLock {
            farmer_id: &farmer_id,
            seed_id: &seed_id,
            amount: &U128(amount),
            increased_power: &U128(increased_seed_power),
            duration: duration_sec,
        }
        .emit();
    }


    pub fn withdraw_seed(&mut self, seed_id: SeedId, amount: Option<U128>) -> Promise {
        require!(self.data().state == RunningState::Running, E004_CONTRACT_PAUSED);
        let farmer_id = env::predecessor_account_id();
        let mut farmer = self.internal_unwrap_farmer(&farmer_id);
        let withdraw_seed = farmer.withdraws.get(&seed_id).unwrap();
        let withdraw_amount: Balance = if let Some(amount) = amount {
            amount.into()
        } else {
            withdraw_seed.amount
        };
        farmer.sub_withdraw_seed(&seed_id, withdraw_amount, self.get_config().delay_withdraw_sec);
        self.internal_set_farmer(&farmer_id, farmer);
        self.transfer_seed_token(&farmer_id, &seed_id, withdraw_amount)
    }

    #[payable]
    pub fn unlock_and_unstake_seed(
        &mut self,
        seed_id: SeedId,
        unlock_amount: U128,
        unstake_amount: U128,
    ) {
        assert_one_yocto();
        require!(self.data().state == RunningState::Running, E004_CONTRACT_PAUSED);

        let unlock_amount: Balance = unlock_amount.into();
        let unstake_amount: Balance = unstake_amount.into();

        let farmer_id = env::predecessor_account_id();

        let mut farmer = self.internal_unwrap_farmer(&farmer_id);
        let mut seed = self.internal_unwrap_seed(&seed_id);

        self.internal_do_farmer_claim(&mut farmer, &mut seed);

        let mut farmer_seed = farmer.seeds.get(&seed_id).unwrap();

        let prev = farmer_seed.get_seed_power();

        let decreased_seed_power = 
        if unlock_amount > 0 {
            farmer_seed.unlock_to_free(unlock_amount)
        } else {
            0
        };
        if unstake_amount > 0 {
            farmer_seed.withdraw_free(unstake_amount);
            farmer.add_withdraw_seed(&seed_id, unstake_amount);
        }

        seed.total_seed_amount -= unstake_amount;
        seed.total_seed_power = seed.total_seed_power - prev + farmer_seed.get_seed_power();

        if farmer_seed.is_empty() {
            farmer.seeds.remove(&seed_id);
            if seed.farmer_count > 0 {
                seed.farmer_count -= 1;
            }
        } else {
            farmer.seeds.insert(&seed_id, &farmer_seed);
        }

        self.update_impacted_seeds(&mut farmer, &seed_id);

        self.internal_set_farmer(&farmer_id, farmer);
        self.internal_set_seed(&seed_id, seed);

        if unlock_amount > 0 {
            Event::SeedUnlock {
                farmer_id: &farmer_id,
                seed_id: &seed_id,
                unlock_amount: &U128(unlock_amount),
                decreased_power: &U128(decreased_seed_power),
                slashed_seed: &U128(0),
            }
            .emit();
        }
    }

    #[payable]
    pub fn force_unlock(
        &mut self,
        seed_id: SeedId,
        unlock_amount: U128,
    ) {
        assert_one_yocto();
        require!(self.data().state == RunningState::Running, E004_CONTRACT_PAUSED);

        let unlock_amount: Balance = unlock_amount.into();

        let farmer_id = env::predecessor_account_id();

        let mut farmer = self.internal_unwrap_farmer(&farmer_id);
        let mut seed = self.internal_unwrap_seed(&seed_id);

        self.internal_do_farmer_claim(&mut farmer, &mut seed);

        let mut farmer_seed = farmer.seeds.get(&seed_id).unwrap();

        let (reduced_seed_power, seed_slashed) = farmer_seed.unlock_to_free_with_slashed(unlock_amount, seed.slash_rate);

        seed.total_seed_amount -= seed_slashed;
        seed.total_seed_power -= reduced_seed_power;

        let slashed_amount = self.data().seeds_slashed.get(&seed_id).unwrap_or(0);
        self.data_mut()
            .seeds_slashed
            .insert(&seed_id, &(slashed_amount + seed_slashed));

        farmer.seeds.insert(&seed_id, &farmer_seed);

        self.update_impacted_seeds(&mut farmer, &seed_id);

        self.internal_set_farmer(&farmer_id, farmer);
        self.internal_set_seed(&seed_id, seed);

        Event::SeedUnlock {
            farmer_id: &farmer_id,
            seed_id: &seed_id,
            unlock_amount: &U128(unlock_amount),
            decreased_power: &U128(reduced_seed_power),
            slashed_seed: &U128(seed_slashed),
        }
        .emit();
    }

    #[private]
    pub fn callback_withdraw_seed(&mut self, seed_id: SeedId, sender_id: AccountId, amount: U128) -> bool {
        require!(
            env::promise_results_count() == 1,
            E001_PROMISE_RESULT_COUNT_INVALID
        );
        let amount: Balance = amount.into();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => {
                // all seed amount goes back to withdraws
               if let Some(mut farmer) = self.internal_get_farmer(&sender_id) {
                    farmer.add_withdraw_seed(&seed_id, amount);
                    self.internal_set_farmer(&sender_id, farmer);
               } else {
                    // if inner farmer not exist, goes to lostfound
                    let seed_amount = self.data().seeds_lostfound.get(&seed_id).unwrap_or(0);
                    self.data_mut()
                        .seeds_lostfound
                        .insert(&seed_id, &(seed_amount + amount));
               }
                Event::SeedWithdraw {
                    farmer_id: &sender_id,
                    seed_id: &seed_id,
                    withdraw_amount: &U128(amount),
                    success: false,
                }
                .emit();
                false
            }
            PromiseResult::Successful(_) => {
                Event::SeedWithdraw {
                    farmer_id: &sender_id,
                    seed_id: &seed_id,
                    withdraw_amount: &U128(amount),
                    success: true,
                }
                .emit();
                true
            }
        }
    }
}

impl Contract {

    fn transfer_seed_token(
        &mut self,
        farmer_id: &AccountId,
        seed_id: &SeedId,
        amount: Balance,
    ) -> Promise {
        let (token, token_id) = parse_seed_id(seed_id);

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
            .then(ext_self::callback_withdraw_seed(
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
            .then(ext_self::callback_withdraw_seed(
                seed_id.clone(),
                farmer_id.clone(),
                amount.into(),
                env::current_account_id(),
                0,
                GAS_FOR_RESOLVE_SEED_TRANSFER,
            ))
        }
    }
}
