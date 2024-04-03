mod setup;
use crate::setup::*;
use std::collections::HashMap;

#[test]
fn test_modify_booster(){
    let e = init_env();
    let users = Users::init(&e);
    let tokens = Tokens::init(&e);

    let inner_id = "0".to_string();
    let seed_id = e.mft_seed_id(&inner_id);

    let booster_id = tokens.love_ref.account_id.to_string();
    let mut affected_seeds = HashMap::new();
    affected_seeds.insert(seed_id.clone(), 100);
    let booster_info = BoosterInfo { booster_decimal: 18, affected_seeds: affected_seeds.clone(), boost_suppress_factor: 1};

    let mut affected_seeds_include_own = HashMap::new();
    affected_seeds_include_own.insert(seed_id.clone(), 100);
    affected_seeds_include_own.insert(booster_id.clone(), 100);
    let booster_info_include_own = BoosterInfo { booster_decimal: 18, affected_seeds: affected_seeds_include_own.clone(), boost_suppress_factor: 1};

    let mut affected_seeds_not_exist = HashMap::new();
    affected_seeds_not_exist.insert("seed".to_string(), 100);
    let booster_info_not_exist = BoosterInfo { booster_decimal: 18, affected_seeds: affected_seeds_not_exist.clone(), boost_suppress_factor: 1};


    let mut affected_seeds_exceed_seed = HashMap::new();
    for i in 0..MAX_NUM_SEEDS_PER_BOOSTER + 1 {
        let temp_seed_id = format!("{}{}", seed_id, i);
        e.create_seed(&e.owner, &temp_seed_id, TOKEN_DECIMALS as u32, None, None).assert_success();
        e.create_farm(&e.owner, &temp_seed_id, &tokens.nref, to_sec(e.current_time()), to_yocto("10")).assert_success();
        affected_seeds_exceed_seed.insert(temp_seed_id, 100);
    }
    let booster_info_exceed_seed = BoosterInfo { booster_decimal: 18, affected_seeds: affected_seeds_exceed_seed.clone(), boost_suppress_factor: 1};

    let mut affected_seeds_exceed_farm = HashMap::new();
    for i in 0..MAX_NUM_SEEDS_PER_BOOSTER {
        let temp_seed_id = format!("{}{}", "seed_id", i);
        e.create_seed(&e.owner, &temp_seed_id, TOKEN_DECIMALS as u32, None, None).assert_success();
        for _ in 0..e.get_config().max_num_farms_per_booster / MAX_NUM_SEEDS_PER_BOOSTER as u32 + 1 {
            e.create_farm(&e.owner, &temp_seed_id, &tokens.nref, to_sec(e.current_time()), to_yocto("10")).assert_success();
            affected_seeds_exceed_farm.insert(temp_seed_id.clone(), 100);
        }
    }
    let booster_info_exceed_farm = BoosterInfo { booster_decimal: 18, affected_seeds: affected_seeds_exceed_farm.clone(), boost_suppress_factor: 1};

    // error scene 
    // 1 : E002_NOT_ALLOWED
    assert_err!(
        e.modify_booster(&users.bob, &booster_id, &booster_info),
        E002_NOT_ALLOWED
    );
    
    // 2 : E301_SEED_NOT_EXIST
    assert_err!(
        e.modify_booster(&e.owner, &booster_id, &booster_info),
        E301_SEED_NOT_EXIST
    );

    e.create_seed(&e.owner, &seed_id, TOKEN_DECIMALS as u32, None, None).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    e.create_seed(&e.owner, &booster_id, TOKEN_DECIMALS as u32, None, None).assert_success();
    assert_seed(e.get_seed(&booster_id), &booster_id, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);

    // 3 : E202_FORBID_SELF_BOOST
    assert_err!(
        e.modify_booster(&e.owner, &booster_id, &booster_info_include_own),
        E202_FORBID_SELF_BOOST
    );

    // 4 : E301_SEED_NOT_EXIST
    assert_err!(
        e.modify_booster(&e.owner, &booster_id, &booster_info_not_exist),
        E301_SEED_NOT_EXIST
    );

    // 5 : E203_EXCEED_FARM_NUM_IN_BOOST
    assert_err!(
        e.modify_booster(&e.owner, &booster_id, &booster_info_exceed_farm),
        E203_EXCEED_FARM_NUM_IN_BOOST
    );

    // 6 : E204_EXCEED_SEED_NUM_IN_BOOSTER
    assert_err!(
        e.modify_booster(&e.owner, &booster_id, &booster_info_exceed_seed),
        E204_EXCEED_SEED_NUM_IN_BOOSTER
    );

    // success
    assert_eq!(e.get_config().booster_seeds.len(), 0);
    e.modify_booster(&e.owner, &booster_id, &booster_info).assert_success();
    assert_eq!(e.get_config().booster_seeds.len(), 1);
    assert_eq!(e.get_config().booster_seeds.get(&booster_id).unwrap().booster_decimal, 18);
    assert_eq!(e.get_config().booster_seeds.get(&booster_id).unwrap().affected_seeds, affected_seeds);
}