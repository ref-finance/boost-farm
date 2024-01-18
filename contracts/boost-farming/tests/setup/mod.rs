#![allow(dead_code)]

use near_sdk::serde_json::json;
use near_sdk::{env, AccountId, Balance, Gas, Timestamp};
use near_sdk_sim::runtime::GenesisConfig;
pub use near_sdk_sim::{
    deploy, init_simulator, to_yocto, ContractAccount, ExecutionResult, UserAccount,
};

use mock_mft::ContractContract as MockMultiFungibleToken;
pub use boost_farming::{ContractContract as FarmingContract,
    MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, NANOS_PER_DAY, DEFAULT_MAX_LOCKING_DURATION_SEC, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC, DEFAULT_MAX_LOCKING_REWARD_RATIO, MIN_LOCKING_REWARD_RATIO, DEFAULT_MAX_NUM_FARMS_PER_SEED, BP_DENOM, MAX_NUM_SEEDS_PER_BOOSTER,
    Seed, SeedFarm, SeedId, FarmId, FarmTerms, FarmStatus, FarmerSeed, RunningState, ImportFarmerInfo, ImportSeedInfo
};

pub use boost_farming::{
    E000_ALREADY_INIT, E001_PROMISE_RESULT_COUNT_INVALID, E002_NOT_ALLOWED, E003_NOT_INIT, E004_CONTRACT_PAUSED, E005_NOT_ALLOWED_ON_CUR_STATE, E006_NOT_IMPLEMENTED, E007_INVALID_OPERATOR,
    E100_ACC_NOT_REGISTERED, E101_INSUFFICIENT_BALANCE, E102_INSUFFICIENT_STORAGE, E103_STILL_HAS_REWARD, E104_STILL_HAS_SEED,
    E200_INVALID_RATIO, E201_INVALID_DURATION, E202_FORBID_SELF_BOOST, E203_EXCEED_FARM_NUM_IN_BOOST, E204_EXCEED_SEED_NUM_IN_BOOSTER, E205_INVALID_SLASH_RATE,
    E300_FORBID_LOCKING, E301_SEED_NOT_EXIST, E302_SEED_ALREADY_EXIST, E303_EXCEED_FARM_NUM_IN_SEED, E304_CAUSE_PRE_UNLOCK, E305_STILL_IN_LOCK, E306_LOCK_AMOUNT_TOO_SMALL, E307_BELOW_MIN_DEPOSIT, E308_INVALID_SEED_ID, E309_NO_NEED_FORCE,
    E401_FARM_NOT_EXIST, E403_FARM_ALREADY_DEPOSIT_REWARD, E404_UNMATCHED_REWARD_TOKEN, E405_FARM_NOT_ENDED, E406_INVALID_FARM_ID,
    E500_INVALID_MSG,
    E600_MFT_INVALID_TOKEN_ID, E601_MFT_CAN_NOT_BE_REWARD
};

mod users;
pub use users::*;
mod tokens;
pub use tokens::*;
mod views;
pub use views::*;
mod utils;
pub use utils::*;
mod owner;
pub use owner::*;
mod management;
pub use management::*;
mod seed;
pub use seed::*;
mod farmer_seed;
pub use farmer_seed::*;
mod farmer_reward;
pub use farmer_reward::*;
mod token_receiver;
pub use token_receiver::*;
mod storage_impl;
pub use storage_impl::*;
mod booster;
pub use booster::*;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    PREV_BOOST_FARMING_WASM_BYTES => "../../releases/boost_farming_032_release.wasm",
    BOOST_FARMING_WASM_BYTES => "../../res/boost_farming.wasm",

    FUNGIBLE_TOKEN_WASM_BYTES => "../../res/mock_ft.wasm",
    MULTI_FUNGIBLE_TOKEN_WASM_BYTES => "../../res/mock_mft.wasm",
}

pub fn previous_boost_farm_wasm_bytes() -> &'static [u8] {
    &PREV_BOOST_FARMING_WASM_BYTES
}

pub fn boost_farm_wasm_bytes() -> &'static [u8] {
    &BOOST_FARMING_WASM_BYTES
}


pub const NEAR: &str = "near";
pub const BOOST_FARMING_ID: &str = "boost_farming.near";
pub const FUNGIBLE_TOKEN_ID: &str = "token.near";
pub const MULTI_FUNGIBLE_TOKEN_ID: &str = "mutlitoken.near";
pub const OWNER_ID: &str = "owner.near";
// pub const OWNER_ID: &str = "ref-dev.testnet";

pub const DEFAULT_GAS: Gas = Gas(Gas::ONE_TERA.0 * 15);
pub const MAX_GAS: Gas = Gas(Gas::ONE_TERA.0 * 300);
pub const TOKEN_DECIMALS: u8 = 24;
pub const TOKEN_TOTAL_SUPPLY: Balance =
    1_000_000_000 * 10u128.pow(TOKEN_DECIMALS as _);

pub const GENESIS_TIMESTAMP: u64 = 1_600_000_000 * 10u64.pow(9);

pub struct Env {
    pub root: UserAccount,
    pub near: UserAccount,
    pub owner: UserAccount,
    pub farming_contract: ContractAccount<FarmingContract>,
    pub multi_fungible_token_contract: ContractAccount<MockMultiFungibleToken>,
}

pub fn to_nano(timestamp: u32) -> Timestamp {
    Timestamp::from(timestamp) * 10u64.pow(9)
}

pub fn to_sec(timestamp: Timestamp) -> u32 {
    (timestamp / 10u64.pow(9)) as u32
}

pub fn init_env() -> Env {
    Env::init_with_contract(&BOOST_FARMING_WASM_BYTES)
}

impl Env {
    pub fn init_with_contract(contract_bytes: &[u8]) -> Self {
        let mut genesis_config = GenesisConfig::default();
        genesis_config.genesis_time = GENESIS_TIMESTAMP;
        genesis_config.block_prod_time = 0;
        
        let root = init_simulator(Some(genesis_config));
        let near = root.create_user(
            AccountId::new_unchecked(NEAR.to_string()),
            to_yocto("1000000"),
        );
        let owner = near.create_user(
            AccountId::new_unchecked(OWNER_ID.to_string()),
            to_yocto("10000"),
        );

        let farming_contract = deploy!(
            contract: FarmingContract,
            contract_id: BOOST_FARMING_ID.to_string(),
            bytes: &contract_bytes,
            signer_account: near,
            deposit: to_yocto("20"),
            gas: DEFAULT_GAS.0,
            init_method: new(
                owner.account_id(),
                AccountId::new_unchecked("ref_exchange".to_string())
            )
        );

        let multi_fungible_token_contract = deploy!(
            contract: MockMultiFungibleToken,
            contract_id: MULTI_FUNGIBLE_TOKEN_ID.to_string(),
            bytes: &MULTI_FUNGIBLE_TOKEN_WASM_BYTES,
            signer_account: near,
            deposit: to_yocto("20"),
            gas: DEFAULT_GAS.0,
            init_method: new(
                "Multi Fungible Token".to_string(),
                "MFT".to_string(),
                TOKEN_DECIMALS
            )
        );

        Self {
            root,
            near,
            owner,
            farming_contract,
            multi_fungible_token_contract
        }
    }

    pub fn storage_deposit_self_to_farming(
        &self,
        user: &UserAccount,
    ) -> ExecutionResult {
        user.call(
            self.farming_contract.account_id(),
            "storage_deposit",
            b"{}",
            MAX_GAS.0,
            to_yocto("0.1"),
        )
    }

    pub fn upgrade_contract(&self, user: &UserAccount, contract_bytes: &[u8]) -> ExecutionResult {
        user
            .create_transaction(account_id(BOOST_FARMING_ID))
            .function_call("upgrade".to_string(), contract_bytes.to_vec(), MAX_GAS.0, 0)
            .submit()
    }

    pub fn skip_time(&self, seconds: u32) {
        self.near.borrow_runtime_mut().cur_block.block_timestamp += to_nano(seconds);
    }

    pub fn current_time(&self) -> u64{
        self.near.borrow_runtime().cur_block.block_timestamp
    }

    pub fn mft_seed_id(&self, token_id: &String) -> String{
        format!("{}@{}", self.multi_fungible_token_contract.user_account.account_id, token_id)
    }
}

pub fn d(value: Balance, decimals: u8) -> Balance {
    value * 10u128.pow(decimals as _)
}
pub fn account_id(account_id: &str) -> AccountId {
    AccountId::new_unchecked(account_id.to_string())
}

pub fn init_token(e: &Env, token_account_id: &AccountId, decimals: u8) -> UserAccount {
    let token = e.near.deploy_and_init(
        &FUNGIBLE_TOKEN_WASM_BYTES,
        token_account_id.clone(),
        "new",
        &json!({
            "name": token_account_id.to_string(),
            "symbol": token_account_id.to_string(),
            "decimals": decimals
        })
        .to_string()
        .into_bytes(),
        to_yocto("10"),
        DEFAULT_GAS.0,
    );

    e.owner.call(
        token_account_id.clone(),
        "storage_deposit",
        &json!({ "account_id": e.farming_contract.account_id() }).to_string().into_bytes(),
        DEFAULT_GAS.0,
        125 * env::STORAGE_PRICE_PER_BYTE,
    )
    .assert_success();
    token
}

#[macro_export]
macro_rules! assert_err{
    (print $exec_func: expr)=>{
        println!("{:?}", $exec_func.promise_errors()[0].as_ref().unwrap().status());
    };
    ($exec_func: expr, $err_info: expr)=>{
        assert!(format!("{:?}", $exec_func.promise_errors()[0].as_ref().unwrap().status()).contains($err_info));
    };
}