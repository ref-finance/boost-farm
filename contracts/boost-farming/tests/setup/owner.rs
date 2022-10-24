use crate::*;


impl Env {
    pub fn grant_next_owner(
        &self, 
        operator: &UserAccount,
        new_owner: &UserAccount,
        deposit: u128
    ) -> ExecutionResult {
        operator
            .function_call(
                self.farming_contract.contract.grant_next_owner(
                    new_owner.account_id(),
                ),
                MAX_GAS.0,
                deposit,
            )
    }

    pub fn accept_next_owner(
        &self, 
        operator: &UserAccount,
        deposit: u128
    ) -> ExecutionResult {
        operator
            .function_call(
                self.farming_contract.contract.accept_next_owner(),
                MAX_GAS.0,
                deposit,
            )
    }

    pub fn confirm_next_owner(
        &self, 
        operator: &UserAccount,
        deposit: u128
    ) -> ExecutionResult {
        operator
            .function_call(
                self.farming_contract.contract.confirm_next_owner(),
                MAX_GAS.0,
                deposit,
            )
    }

    pub fn cancel_next_owner(
        &self, 
        operator: &UserAccount,
        deposit: u128
    ) -> ExecutionResult {
        operator
            .function_call(
                self.farming_contract.contract.cancel_next_owner(),
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
}