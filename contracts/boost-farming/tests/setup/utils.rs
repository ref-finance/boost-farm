use crate::*;
use near_sdk::serde_json::{json, Value};
use near_sdk::{Balance, Timestamp};

uint::construct_uint!(
    pub struct U256(4);
);

pub fn assert_seed(seed: Value, seed_id: &SeedId, seed_decimal: u32, next_index: u32, total_seed_amount: Balance, total_seed_power: Balance, min_deposit: Balance, slash_rate: u32, min_locking_duration_sec: u32){
    assert_eq!(json!(seed_id.clone()), *seed.get("seed_id").unwrap());
    assert_eq!(json!(seed_decimal), *seed.get("seed_decimal").unwrap());
    assert_eq!(json!(next_index), *seed.get("next_index").unwrap());
    assert_eq!(total_seed_amount.to_string(), *seed.get("total_seed_amount").unwrap());
    assert_eq!(total_seed_power.to_string(), *seed.get("total_seed_power").unwrap());
    assert_eq!(min_deposit.to_string(), *seed.get("min_deposit").unwrap());
    assert_eq!(json!(slash_rate), *seed.get("slash_rate").unwrap());
    assert_eq!(json!(min_locking_duration_sec), *seed.get("min_locking_duration_sec").unwrap());
}

pub fn assert_farm_info(seed_farm: SeedFarm, farm_id: &FarmId, reward_token: &UserAccount, start_at: u32, daily_reward: Balance) {
    assert_eq!(seed_farm.farm_id, farm_id.clone());
    assert_eq!(seed_farm.terms.reward_token, reward_token.account_id());
    assert_eq!(seed_farm.terms.start_at, start_at);
    assert_eq!(seed_farm.terms.daily_reward, daily_reward);
}

pub fn assert_farm_detail(seed_farm: SeedFarm, total_reward: Balance, distributed_at: Timestamp, distributed_reward: Balance, claimed_reward: Balance, amount_of_beneficiary: Balance, amount_of_withdrew_beneficiary: Balance, status: Option<FarmStatus>){
    assert_eq!(seed_farm.total_reward, total_reward);
    assert_eq!(seed_farm.distributed_at, distributed_at);
    assert_eq!(seed_farm.distributed_reward, distributed_reward);
    assert_eq!(seed_farm.claimed_reward, claimed_reward);
    assert_eq!(seed_farm.amount_of_beneficiary, amount_of_beneficiary);
    assert_eq!(seed_farm.amount_of_withdrew_beneficiary, amount_of_withdrew_beneficiary);
    assert_eq!(format!("{:?}", seed_farm.status), format!("{:?}", status));
}

pub fn assert_user_seed_info(farmer_seed: Value, free_amount: Balance, locked_amount: Balance, x_locked_amount: Balance, unlock_timestamp: u64, duration_sec: u32){
    assert_eq!(free_amount.to_string(), *farmer_seed.get("free_amount").unwrap());
    assert_eq!(locked_amount.to_string(), *farmer_seed.get("locked_amount").unwrap());
    assert_eq!(x_locked_amount.to_string(), *farmer_seed.get("x_locked_amount").unwrap());
    assert_eq!(unlock_timestamp.to_string(), *farmer_seed.get("unlock_timestamp").unwrap());
    assert_eq!(json!(duration_sec), *farmer_seed.get("duration_sec").unwrap());
}

pub fn u128_ratio(a: u128, num: u128, denom: u128) -> Balance {
    (U256::from(a) * U256::from(num) / U256::from(denom)).as_u128()
}