use crate::*;
use near_sdk::json_types::U128;

impl Env {

    pub fn lock_free_seed(
        &self,
        operator: &UserAccount,
        seed_id: &SeedId, 
        duration_sec: u32, 
        amount: u128
    ) -> ExecutionResult {
        operator
            .function_call(
                self.farming_contract.contract.lock_free_seed(
                    seed_id.clone(),
                    duration_sec,
                    Some(U128(amount))
                ),
                MAX_GAS.0,
                1,
            )
    }

    pub fn unlock_and_withdraw_seed(
        &self,
        operator: &UserAccount,
        seed_id: &SeedId,
        unlock_amount: u128,
        withdraw_amount: u128,
    ) -> ExecutionResult {
        operator
            .function_call(
                self.farming_contract.contract.unlock_and_withdraw_seed(
                    seed_id.clone(),
                    U128(unlock_amount),
                    U128(withdraw_amount)
                ),
                MAX_GAS.0,
                1,
            )
    }

    pub fn force_unlock(
        &self,
        operator: &UserAccount,
        seed_id: &SeedId,
        unlock_amount: u128,
    ) -> ExecutionResult {
        operator
            .function_call(
                self.farming_contract.contract.force_unlock(
                    seed_id.clone(),
                    U128(unlock_amount),
                ),
                MAX_GAS.0,
                1,
            )
    }

    
}