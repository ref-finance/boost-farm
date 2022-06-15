use crate::*;
use near_sdk::json_types::U128;
use near_sdk::Balance;

impl Env {
    pub fn create_seed(
        &self,
        operator: &UserAccount,
        seed_id: &SeedId,
        seed_decimal:u32,
        min_deposit: Option<U128>,
        min_locking_duration_sec: Option<u32>,
    ) -> ExecutionResult {
        operator
            .function_call(
                self.farming_contract.contract.create_seed(
                    seed_id.clone(),
                    seed_decimal,
                    min_deposit,
                    min_locking_duration_sec
                ),
                DEFAULT_GAS.0,
                1,
            )
    }
    
    pub fn create_farm(
        &self,
        operator: &UserAccount,
        seed_id: &SeedId,
        reward_token: &UserAccount,
        start_at: u32,
        daily_reward: Balance
    ) -> ExecutionResult {
        operator
            .function_call(
                self.farming_contract.contract.create_farm(
                    seed_id.clone(),
                    FarmTerms{
                        reward_token: reward_token.account_id(),
                        start_at,
                        daily_reward
                    },
                ),
                MAX_GAS.0,
                1,
            )
    }

    pub fn cancel_farm(
        &self,
        operator: &UserAccount,
        farm_id: &String
    ) -> ExecutionResult {
        operator
        .function_call(
            self.farming_contract.contract.cancel_farm(
                farm_id.clone()
            ),
            DEFAULT_GAS.0,
            1,
        )
    }

    pub fn remove_farm_from_seed(
        &self,
        operator: &UserAccount,
        farm_id: &String
    ) -> ExecutionResult {
        operator
            .function_call(
                self.farming_contract.contract.remove_farm_from_seed(
                    farm_id.clone(),
                ),
                MAX_GAS.0,
                1,
            )
    }
}