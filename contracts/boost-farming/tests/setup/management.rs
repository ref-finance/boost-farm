use crate::*;
use near_sdk::json_types::U128;


impl Env {

    pub fn modify_daily_reward(
        &self,
        operator: &UserAccount,
        farm_id: &FarmId, 
        daily_reward: u128,
        deposit: u128
    ) -> ExecutionResult {
        operator
        .function_call(
            self.farming_contract.contract.modify_daily_reward(
                farm_id.clone(),
                U128(daily_reward)
            ),
            MAX_GAS.0,
            deposit,
        )
    }

    pub fn modify_locking_policy(
        &self,
        operator: &UserAccount,
        max_duration: u32, 
        max_ratio: u32,
        deposit: u128
    ) -> ExecutionResult {
        operator
        .function_call(
            self.farming_contract.contract.modify_locking_policy(
                max_duration,
                max_ratio
            ),
            MAX_GAS.0,
            deposit,
        )
    }

    pub fn modify_max_farm_num_per_seed(
        &self,
        operator: &UserAccount,
        max_num: u32,
        deposit: u128
    ) -> ExecutionResult {
        operator
            .function_call(
                self.farming_contract.contract.modify_max_farm_num_per_seed(
                    max_num
                ),
                MAX_GAS.0,
                deposit,
            )
    }


    pub fn modify_default_slash_rate(
        &self,
        operator: &UserAccount,
        slash_rate: u32,
        deposit: u128
    ) -> ExecutionResult {
        operator
            .function_call(
                self.farming_contract.contract.modify_default_slash_rate(
                    slash_rate
                ),
                MAX_GAS.0,
                deposit,
            )
    }

    pub fn modify_seed_min_deposit(
        &self,
        operator: &UserAccount,
        seed_id: &String, 
        min_deposit: u128,
        deposit: u128
    ) -> ExecutionResult {
        operator
            .function_call(
                self.farming_contract.contract.modify_seed_min_deposit(
                    seed_id.clone(),
                    U128(min_deposit)
                ),
                MAX_GAS.0,
                deposit,
            )
    }

    pub fn modify_seed_min_locking_duration(
        &self,
        operator: &UserAccount,
        seed_id: &String, 
        min_locking_duration_sec: u32,
        deposit: u128
    ) -> ExecutionResult {
        operator
            .function_call(
                self.farming_contract.contract.modify_seed_min_locking_duration(
                    seed_id.clone(),
                    min_locking_duration_sec
                ),
                MAX_GAS.0,
                deposit,
            )
    }

    pub fn modify_seed_slash_rate(
        &self,
        operator: &UserAccount,
        seed_id: &String, 
        slash_rate: u32,
        deposit: u128
    ) -> ExecutionResult {
        operator
            .function_call(
                self.farming_contract.contract.modify_seed_slash_rate(
                    seed_id.clone(),
                    slash_rate
                ),
                MAX_GAS.0,
                deposit,
            )
    }

    pub fn withdraw_seed_slashed(
        &self,
        operator: &UserAccount,
        seed_id: &SeedId,
        deposit: u128
    ) -> ExecutionResult {
        operator
        .function_call(
            self.farming_contract.contract.withdraw_seed_slashed(
                seed_id.clone(),
            ),
            MAX_GAS.0,
            deposit,
        )
    }

    pub fn return_seed_lostfound(
        &self,
        operator: &UserAccount,
        farmer: &UserAccount, 
        seed_id: &SeedId, 
        amount: u128,
        deposit: u128
    ) -> ExecutionResult {
        operator
        .function_call(
            self.farming_contract.contract.return_seed_lostfound(
                farmer.account_id(),
                seed_id.clone(),
                U128(amount)
            ),
            MAX_GAS.0,
            deposit,
        )
    }

    pub fn withdraw_from_beneficiary_account(
        &self,
        operator: &UserAccount,
        farm_id: &FarmId, 
        deposit: u128
    ) -> ExecutionResult {
        operator
        .function_call(
            self.farming_contract.contract.withdraw_from_beneficiary_account(
                farm_id.clone()
            ),
            MAX_GAS.0,
            deposit,
        )
    }

    pub fn withdraw_from_outdated_beneficiary_account(
        &self,
        operator: &UserAccount,
        farm_id: &FarmId, 
        deposit: u128
    ) -> ExecutionResult {
        operator
        .function_call(
            self.farming_contract.contract.withdraw_from_outdated_beneficiary_account(
                farm_id.clone()
            ),
            MAX_GAS.0,
            deposit,
        )
    }

    pub fn withdraw_from_undistributed_reward(
        &self,
        operator: &UserAccount,
        farm_id: &FarmId, 
        amount: u128,
        deposit: u128
    ) -> ExecutionResult {
        operator
        .function_call(
            self.farming_contract.contract.withdraw_from_undistributed_reward(
                farm_id.clone(),
                U128(amount)
            ),
            MAX_GAS.0,
            deposit,
        )
    }

}