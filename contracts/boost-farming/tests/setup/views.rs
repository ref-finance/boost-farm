use crate::*;
use near_sdk::serde::{Deserialize, Serialize};
use std::collections::HashMap;
use near_sdk::serde_json::Value;
use near_sdk::json_types::{U64, U128};
use near_sdk::AccountId;

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Metadata {
    pub version: String,
    pub owner_id: AccountId,
    pub next_owner_id: Option<AccountId>,
    pub next_owner_accept_deadline: Option<u64>,
    pub state: RunningState,
    pub operators: Vec<AccountId>,
    pub farmer_count: U64,
    pub farm_count: U64,
    pub outdated_farm_count: U64,
    pub seed_count: U64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct StorageReport {
    pub storage: U64,
    pub locking_near: U128,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct BoosterInfo {
    pub booster_decimal: u32,
    pub affected_seeds: HashMap<SeedId, u32>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Config {
    pub delay_withdraw_sec: u32,
    pub seed_slash_rate: u32,
    pub booster_seeds: HashMap<SeedId, BoosterInfo>,
    pub max_num_farms_per_booster: u32,
    pub max_num_farms_per_seed: u32,
    pub maximum_locking_duration_sec: u32,
    pub max_locking_multiplier: u32,
}

impl Env {

    pub fn get_metadata(&self) -> Metadata{
        self.owner
        .view_method_call(
            self.farming_contract.contract.get_metadata()
        ).unwrap_json::<Metadata>()
    }

    pub fn get_config(&self) -> Config{
        self.owner
        .view_method_call(
            self.farming_contract.contract.get_config()
        ).unwrap_json::<Config>()
    }

    pub fn get_contract_storage_report(&self) -> StorageReport{
        self.owner
        .view_method_call(
            self.farming_contract.contract.get_contract_storage_report()
        ).unwrap_json::<StorageReport>()
    }

    pub fn get_seed(&self, seed_id: &SeedId) -> Value {
        self.owner
            .view_method_call(
                self.farming_contract.contract.get_seed(seed_id.clone())
            ).unwrap_json_value()
    }
    
    pub fn list_seeds_info(&self) -> Value {
        self.owner
            .view_method_call(
                self.farming_contract.contract.list_seeds_info(None, None)
            ).unwrap_json_value()
    }

    pub fn get_farm(&self, farm_id: &FarmId) -> SeedFarm {
        let seed_farm: Option<SeedFarm> = self.owner
            .view_method_call(
                self.farming_contract.contract.get_farm(farm_id.clone())
            ).unwrap_json();
        seed_farm.unwrap()
    }
    
    pub fn list_seed_farms(&self, seed_id: &SeedId) -> Vec<SeedFarm> {
        self.owner
            .view_method_call(
                self.farming_contract.contract.list_seed_farms(seed_id.clone())
            ).unwrap_json::<Vec<SeedFarm>>()
    }

    pub fn get_unclaimed_rewards(
        &self,
        farmer: &UserAccount,
        seed_id: &SeedId,
        reward_token: &UserAccount,
    ) -> u128 {
        let rewards: HashMap<AccountId, U128> = self.owner
            .view_method_call(
                self.farming_contract.contract.get_unclaimed_rewards(farmer.account_id(), seed_id.clone())
            ).unwrap_json();
        rewards.get(&reward_token.account_id()).unwrap_or_else(||{ &U128(0) }).0
    }

    pub fn get_farmer_seed(&self, user: &UserAccount, seed_id: &SeedId) -> Value{
        self.owner
        .view_method_call(
            self.farming_contract.contract.get_farmer_seed(user.account_id(), seed_id.clone())
        ).unwrap_json_value()
    }

    pub fn list_farmer_seeds(&self, user: &UserAccount) -> Value{
        self.owner
        .view_method_call(
            self.farming_contract.contract.list_farmer_seeds(user.account_id(), None, None)
        ).unwrap_json_value()
    }

    pub fn get_farmer_reward(&self, user: &UserAccount, token: &UserAccount) -> u128{
        let reward: U128 = self.owner
        .view_method_call(
            self.farming_contract.contract.get_farmer_reward(user.account_id(), token.account_id())
        ).unwrap_json();
        reward.0
    }

    pub fn get_farmer_withdraw(&self, user: &UserAccount, seed_id: &SeedId) -> Value{
        self.owner
        .view_method_call(
            self.farming_contract.contract.get_farmer_withdraw(user.account_id(), seed_id.clone())
        ).unwrap_json_value()
    }

    pub fn list_farmer_rewards(&self, user: &UserAccount) -> HashMap<AccountId, U128> {
        self.owner
        .view_method_call(
            self.farming_contract.contract.list_farmer_rewards(user.account_id())
        ).unwrap_json::<HashMap<AccountId, U128>>()
    }

    pub fn get_outdated_farm(&self, farm_id: &FarmId) -> SeedFarm{
        self.owner
        .view_method_call(
            self.farming_contract.contract.get_outdated_farm(farm_id.clone())
        ).unwrap_json::<SeedFarm>()
    }

    pub fn list_outdated_farms(&self) -> Vec<SeedFarm> {
        self.owner
        .view_method_call(
            self.farming_contract.contract.list_outdated_farms(None, None)
        ).unwrap_json::<Vec<SeedFarm>>()
    }

    pub fn list_slashed(&self) -> HashMap<SeedId, U128>{
        self.owner
        .view_method_call(
            self.farming_contract.contract.list_slashed(None, None)
        ).unwrap_json::<HashMap<SeedId, U128>>()
    }

    pub fn list_lostfound(&self) -> HashMap<SeedId, U128>{
        self.owner
        .view_method_call(
            self.farming_contract.contract.list_lostfound(None, None)
        ).unwrap_json::<HashMap<SeedId, U128>>()
    }
}