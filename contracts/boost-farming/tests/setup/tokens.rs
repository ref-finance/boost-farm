use crate::*;
pub use near_sdk_sim::UserAccount;
use near_sdk::serde_json::json;
use near_sdk::json_types::U128;
use near_sdk::Balance;
use near_sdk::env;

pub struct Tokens {
    pub wnear: UserAccount,
    pub nref: UserAccount,
    pub neth: UserAccount,
    pub ndai: UserAccount,
    pub nusdt: UserAccount,
    pub nusdc: UserAccount,
    pub love_ref: UserAccount,
    pub love_sushi: UserAccount,
}

impl Tokens {
    pub fn init(e: &Env) -> Self {
        Self {
            wnear: init_token(e, &account_id("wrap.near"), 24),
            nref: init_token(e, &account_id("ref.near"), 18),
            neth: init_token(e, &account_id("neth.near"), 18),
            ndai: init_token(e, &account_id("dai.near"), 18),
            nusdt: init_token(e, &account_id("nusdt.near"), 6),
            nusdc: init_token(e, &account_id("nusdc.near"), 6),
            love_ref: init_token(e, &account_id("love_ref.near"), 24),
            love_sushi: init_token(e, &account_id("love_sushi.near"), 24),
        }
    }
}

impl Env {

    pub fn ft_storage_deposit(
        &self,
        user: &UserAccount,
        token: &UserAccount, 
    ){
        user.call(
            token.account_id(),
            "storage_deposit",
            b"{}", 
            DEFAULT_GAS.0,
            125 * env::STORAGE_PRICE_PER_BYTE,
        )
        .assert_success();
    }

    pub fn ft_mint(&self, token: &UserAccount, receiver: &UserAccount, amount: Balance) {
        self.owner
            .call(
                token.account_id.clone(),
                "mint",
                &json!({
                    "account_id": receiver.account_id(),
                    "amount": U128::from(amount),
                })
                .to_string()
                .into_bytes(),
                DEFAULT_GAS.0,
                0,
            )
            .assert_success();
    }

    pub fn ft_balance_of(&self, token: &UserAccount, user_account: &UserAccount) -> u128{
        let amount: U128 = self.owner
            .view(
                token.account_id.clone(),
                "ft_balance_of",
                &json!({
                    "account_id": user_account.account_id()
                }).to_string().into_bytes()
            ).unwrap_json();
        amount.0
    }

    pub fn mft_storage_deposit(
        &self,
        token_id: &String, 
        user: &UserAccount,
    ){
        self.owner
            .function_call(
                self.multi_fungible_token_contract.contract.mft_register(token_id.clone(), user.account_id()),
                DEFAULT_GAS.0,
                125 * env::STORAGE_PRICE_PER_BYTE,
            ).assert_success();
    }

    pub fn mft_unregister(
        &self,
        token_id: &String, 
        user: &UserAccount,
    ){
        user
            .function_call(
                self.multi_fungible_token_contract.contract.mft_unregister(token_id.clone(), Some(true)),
                DEFAULT_GAS.0,
                1,
            ).assert_success();
    }

    pub fn mft_mint(&self, inner_id: &String, user: &UserAccount, amount: Balance) {
        self.owner
            .function_call(
                self.multi_fungible_token_contract.contract.mint(inner_id.clone(), user.account_id(), U128(amount)),
                DEFAULT_GAS.0,
                0,
            ).assert_success();
    }

    pub fn mft_balance_of(&self, user_account: &UserAccount, token_id: &String) -> u128{
        let amount: U128 = self.owner
            .view_method_call(
                self.multi_fungible_token_contract.contract.mft_balance_of(token_id.clone(), user_account.account_id.clone())
            ).unwrap_json();
        amount.0
    }
}

