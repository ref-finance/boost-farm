mod setup;
use crate::setup::*;
use std::collections::HashMap;
use near_sdk::AccountId;
use near_sdk::json_types::U128;


#[test]
fn test_set_owner(){
    let e = init_env();
    let users = Users::init(&e);
    assert_eq!(e.get_metadata().owner_id, e.owner.account_id());
    
    // error scene 
    // 1 : Requires attached deposit of exactly 1 yoctoNEAR
    assert_err!(
        e.set_owner(&users.alice, &e.owner, 0),
        "Requires attached deposit of exactly 1 yoctoNEAR"
    );

    // 2: E002_NOT_ALLOWED
    assert_err!(
        e.set_owner(&users.alice, &e.owner, 1),
        E002_NOT_ALLOWED
    );

    // success
    e.set_owner(&e.owner, &users.alice, 1).assert_success();
    assert_eq!(e.get_metadata().owner_id, users.alice.account_id());
}

#[test]
fn test_pause_resume(){
    let e = init_env();
    let users = Users::init(&e);
    assert_eq!(e.get_metadata().state, RunningState::Running);
    
    // error scene 
    // 1 : Requires attached deposit of exactly 1 yoctoNEAR
    assert_err!(
        e.pause_contract(&users.alice, 0),
        "Requires attached deposit of exactly 1 yoctoNEAR"
    );

    // 2: E002_NOT_ALLOWED
    assert_err!(
        e.pause_contract(&users.alice, 1),
        E002_NOT_ALLOWED
    );

    // success
    e.pause_contract(&e.owner, 1).assert_success();
    assert_eq!(e.get_metadata().state, RunningState::Paused);

    let inner_id = "0".to_string();
    let seed_id = e.mft_seed_id(&inner_id);
    assert_err!(
        e.create_seed(&e.owner, &seed_id, TOKEN_DECIMALS as u32, None, None),
        E004_CONTRACT_PAUSED
    );

    // success
    e.resume_contract(&e.owner, 1).assert_success();
    assert_eq!(e.get_metadata().state, RunningState::Running);

    e.create_seed(&e.owner, &seed_id, TOKEN_DECIMALS as u32, None, None).assert_success();
}

#[test]
fn test_operators(){
    let e = init_env();
    let users = Users::init(&e);

    // error scene 
    // 1 : extend with Requires attached deposit of exactly 1 yoctoNEAR
    assert_err!(
        e.extend_operators(&users.alice, vec![], 0),
        "Requires attached deposit of exactly 1 yoctoNEAR"
    );

    // 2 : extend with E002_NOT_ALLOWED
    assert_err!(
        e.extend_operators(&users.alice, vec![], 1),
        E002_NOT_ALLOWED
    );

    // 3 : remove with Requires attached deposit of exactly 1 yoctoNEAR
    assert_err!(
        e.remove_operators(&users.alice, vec![], 0),
        "Requires attached deposit of exactly 1 yoctoNEAR"
    );

    // 4 : remove with E002_NOT_ALLOWED
    assert_err!(
        e.remove_operators(&users.alice, vec![], 1),
        E002_NOT_ALLOWED
    );

    // success
    assert_eq!(e.get_metadata().operators, vec![]);
    e.set_owner(&e.owner, &users.alice, 1).assert_success();
    e.extend_operators(&users.alice, vec![&users.bob], 1).assert_success();
    assert_eq!(e.get_metadata().operators, vec![users.bob.account_id()]);
    e.remove_operators(&users.alice, vec![&users.bob], 1).assert_success();
    assert_eq!(e.get_metadata().operators, vec![]);
}

#[test]
fn test_pour() {
    let e = init_env();
    let users = Users::init(&e);
    e.pause_contract(&e.owner, 1).assert_success();

    let prev_storage = e.get_contract_storage_report().storage.0;

    let mut rewards = HashMap::new();
    let mut seeds = HashMap::new();
    rewards.insert(AccountId::new_unchecked("token01".to_string()), U128(1));
    seeds.insert("seed01".to_string(), U128(1));
    seeds.insert("seed02".to_string(), U128(1));

    let a = ImportFarmerInfo {
        farmer_id: AccountId::new_unchecked("farmer01.near".to_string()),
        rewards: rewards.clone(),
        seeds: seeds.clone(),
    };
    let b = ImportFarmerInfo {
        farmer_id: AccountId::new_unchecked("farmer02.near".to_string()),
        rewards: HashMap::new(),
        seeds: HashMap::new(),
    };

    e.pour_farmers(&e.owner, vec![a, b]).assert_success();
    assert_eq!(e.get_metadata().farmer_count.0, 2);

    println!("storage increase: {} bytes", e.get_contract_storage_report().storage.0 - prev_storage);

    let seed_info_01 = ImportSeedInfo {
        seed_id: "seed01".to_string(),
        seed_decimal: 24,
        amount: U128(1),
        min_deposit: U128(1000000),
    };
    let seed_info_02 = ImportSeedInfo {
        seed_id: "seed02".to_string(),
        seed_decimal: 24,
        amount: U128(1),
        min_deposit: U128(1000000),
    };
    e.pour_seeds(&e.owner, vec![seed_info_01, seed_info_02]).assert_success();
    assert_eq!(e.get_metadata().seed_count.0, 2);


    e.resume_contract(&e.owner, 1).assert_success();


    assert_err!(
        e.pour_farmers(&e.owner, vec![ImportFarmerInfo {
            farmer_id: AccountId::new_unchecked("farmer03.near".to_string()),
            rewards: HashMap::new(),
            seeds: HashMap::new(),
        }]),
        E005_NOT_ALLOWED_ON_CUR_STATE
    );

    // RUSTFLAGS="-C link-arg=-s" cargo test -p boost-farming test_pour -- --nocapture

}