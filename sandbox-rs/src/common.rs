
use serde_json::json;
use serde::{Deserialize, Serialize};
use workspaces::AccountId;


#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
pub enum RunningState {
    Running, Paused
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub version: String,
    pub owner_id: AccountId,
    pub state: RunningState,
    pub operators: Vec<AccountId>,
    pub farmer_count: String,
    pub farm_count: String,
    pub outdated_farm_count: String,
    pub seed_count: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ImportSeedInfo {
    pub seed_id: String,
    pub seed_decimal: u32,
    pub amount: String,
    pub min_deposit: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ImportFarmerInfo {
    pub farmer_id: AccountId,
    pub rewards: HashMap<AccountId, String>,
    pub seeds: HashMap<SeedId, String>,
}