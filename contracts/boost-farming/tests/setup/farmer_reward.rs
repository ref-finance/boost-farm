use crate::*;
use near_sdk::json_types::U128;


impl Env {
    pub fn claim_reward_by_seed(
        &self,
        operator: &UserAccount,
        seed_id: &SeedId
    ) -> ExecutionResult {
        operator
            .function_call(
                self.farming_contract.contract.claim_reward_by_seed(
                    seed_id.clone(),
                ),
                DEFAULT_GAS.0,
                0,
            )
    }

    pub fn withdraw_reward(
        &self,
        operator: &UserAccount,
        token_id: &UserAccount,
        amount: Option<u128>
    )
    -> ExecutionResult {
        operator
            .function_call(
                self.farming_contract.contract.withdraw_reward(
                    token_id.account_id(),
                    if let Some(amount) = amount { Some(U128(amount)) } else { None }
                ),
                MAX_GAS.0,
                0,
            )
    }
}