use crate::*;
use near_sdk::serde_json::json;
use near_sdk::json_types::U128;
use near_sdk::Balance;

impl Env {
    pub fn deposit_reward(
        &self,
        token: &UserAccount,
        user: &UserAccount,
        amount: Balance,
        farm_id: &String,
    ) -> ExecutionResult {
        user.call(
            token.account_id.clone(),
            "ft_transfer_call",
            &json!({
                "receiver_id": self.farming_contract.user_account.account_id(),
                "amount": U128::from(amount),
                "msg": format!("{{\"Reward\": {{\"farm_id\": \"{}\"}}}}", farm_id),
            })
            .to_string()
            .into_bytes(),
            MAX_GAS.0,
            1,
        )
    }

    pub fn mft_stake_free_seed(
        &self,
        user: &UserAccount,
        token_id: &String,
        amount: Balance,
    ) -> ExecutionResult {
        user.call(
            self.multi_fungible_token_contract.account_id(),
            "mft_transfer_call",
            &json!({
                "token_id": token_id.clone(),
                "receiver_id": self.farming_contract.user_account.account_id(),
                "amount": U128::from(amount),
                "msg": "\"Free\"",
            })
            .to_string()
            .into_bytes(),
            MAX_GAS.0,
            1,
        )
    }

    pub fn mft_stake_lock_seed(
        &self,
        token_id: &String,
        user: &UserAccount,
        amount: Balance,
        duration_sec: u32,
    ) -> ExecutionResult {
        user.call(
            self.multi_fungible_token_contract.account_id(),
            "mft_transfer_call",
            &json!({
                "token_id": token_id.clone(),
                "receiver_id": self.farming_contract.user_account.account_id(),
                "amount": U128::from(amount),
                "msg": format!("{{\"Lock\": {{\"duration_sec\": {}}}}}", duration_sec),
            })
            .to_string()
            .into_bytes(),
            MAX_GAS.0,
            1,
        )
    }

    pub fn ft_stake_free_seed(
        &self,
        user: &UserAccount,
        token: &UserAccount,
        amount: Balance,
    ) -> ExecutionResult {
        user.call(
            token.account_id(),
            "ft_transfer_call",
            &json!({
                "receiver_id": self.farming_contract.user_account.account_id(),
                "amount": U128::from(amount),
                "msg": "\"Free\"",
            })
            .to_string()
            .into_bytes(),
            MAX_GAS.0,
            1,
        )
    }

    pub fn ft_stake_lock_seed(
        &self,
        user: &UserAccount,
        token: &UserAccount,
        amount: Balance,
        duration_sec: u32,
    ) -> ExecutionResult {
        user.call(
            token.account_id(),
            "ft_transfer_call",
            &json!({
                "receiver_id": self.farming_contract.user_account.account_id(),
                "amount": U128::from(amount),
                "msg": format!("{{\"Lock\": {{\"duration_sec\": {}}}}}", duration_sec),
            })
            .to_string()
            .into_bytes(),
            MAX_GAS.0,
            1,
        )
    }
}