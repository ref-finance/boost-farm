use crate::*;


impl Env {
    pub fn set_owner(
        &self, 
        operator: &UserAccount,
        new_owner: &UserAccount,
        deposit: u128
    ) -> ExecutionResult {
        operator
            .function_call(
                self.farming_contract.contract.set_owner(
                    new_owner.account_id(),
                ),
                MAX_GAS.0,
                deposit,
            )
    }

    pub fn pause_contract(
        &self, 
        operator: &UserAccount,
        deposit: u128
    ) -> ExecutionResult {
        operator
            .function_call(
                self.farming_contract.contract.pause_contract(),
                MAX_GAS.0,
                deposit,
            )
    }

    pub fn resume_contract(
        &self, 
        operator: &UserAccount,
        deposit: u128
    ) -> ExecutionResult {
        operator
            .function_call(
                self.farming_contract.contract.resume_contract(),
                MAX_GAS.0,
                deposit,
            )
    }

    pub fn extend_operators(
        &self, 
        operator: &UserAccount,
        operators: Vec<&UserAccount>,
        deposit: u128
    ) -> ExecutionResult {
        let ops = operators.iter().map(|v| v.account_id()).collect::<Vec<_>>();
        operator
        .function_call(
            self.farming_contract.contract.extend_operators(
                ops,
            ),
            MAX_GAS.0,
            deposit,
        )
    }

    pub fn remove_operators(
        &self, 
        operator: &UserAccount,
        operators: Vec<&UserAccount>,
        deposit: u128
    ) -> ExecutionResult {
        let ops = operators.iter().map(|v| v.account_id()).collect::<Vec<_>>();
        operator
        .function_call(
            self.farming_contract.contract.remove_operators(
                ops,
            ),
            MAX_GAS.0,
            deposit,
        )
    }

    pub fn pour_farmers(
        &self, 
        operator: &UserAccount,
        farmers: Vec<ImportFarmerInfo>
    ) -> ExecutionResult {
        operator
            .function_call(
                self.farming_contract.contract.pour_farmers(
                    farmers,
                ),
                MAX_GAS.0,
                1,
            )
    }

    pub fn pour_seeds(
        &self, 
        operator: &UserAccount,
        seeds: Vec<ImportSeedInfo>
    ) -> ExecutionResult {
        operator
            .function_call(
                self.farming_contract.contract.pour_seeds(
                    seeds,
                ),
                MAX_GAS.0,
                1,
            )
    }
}