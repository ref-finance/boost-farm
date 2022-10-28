mod setup;
use crate::setup::*;
use std::collections::HashMap;

#[test]
fn test_create_seed(){
    let e = init_env();
    let users = Users::init(&e);

    let inner_id = "0".to_string();
    let seed_id = e.mft_seed_id(&inner_id);

    // success
    e.create_seed(&e.owner, &seed_id, TOKEN_DECIMALS as u32, None, None).assert_success();

    // error scene
    // 1 : E002_NOT_ALLOWED
    assert_err!(
        e.create_seed(&users.alice, &seed_id, TOKEN_DECIMALS as u32, None, None),
        E002_NOT_ALLOWED
    );

    // 2 : E302_SEED_ALREADY_EXIST
    assert_err!(
        e.create_seed(&e.owner, &seed_id, TOKEN_DECIMALS as u32, None, None),
        E302_SEED_ALREADY_EXIST
    );
}

#[test]
fn test_create_farm(){
    let e = init_env();
    let users = Users::init(&e);
    let tokens = Tokens::init(&e);

    let inner_id = "0".to_string();
    let seed_id = e.mft_seed_id(&inner_id);
    let farm_id = format!("{}#{}", seed_id, 0);

    e.create_seed(&e.owner, &seed_id, TOKEN_DECIMALS as u32, None, None).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);

    // success
    let start_at = e.current_time();
    e.create_farm(&e.owner, &seed_id, &tokens.nref, to_sec(start_at), to_yocto("10")).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 1, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    assert_farm_info(e.get_farm(&farm_id), &farm_id, &tokens.nref, to_sec(start_at), to_yocto("10"));
    assert_farm_detail(e.get_farm(&farm_id), 0, start_at, 0, 0, 0, 0, Some(FarmStatus::Created));

    // error scene
    // 1 : E002_NOT_ALLOWED
    assert_err!(
        e.create_farm(&users.alice, &seed_id, &tokens.nref, to_sec(start_at), to_yocto("10")),
        E002_NOT_ALLOWED
    );

    // 2 : E301_SEED_NOT_EXIST
    assert_err!(
        e.create_farm(&e.owner, &"seed_id".to_string(), &tokens.nref, to_sec(start_at), to_yocto("10")),
        E301_SEED_NOT_EXIST
    );

    for _ in 0..DEFAULT_MAX_NUM_FARMS_PER_SEED - 1 {
        e.create_farm(&e.owner, &seed_id, &tokens.nref, to_sec(start_at), to_yocto("10")).assert_success();
    }
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, DEFAULT_MAX_NUM_FARMS_PER_SEED, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    assert_eq!(e.get_metadata().farm_count.0, DEFAULT_MAX_NUM_FARMS_PER_SEED as u64);

    // 3 : E303_EXCEED_FARM_NUM_IN_SEED
    assert_err!(
        e.create_farm(&e.owner, &seed_id, &tokens.nref, to_sec(start_at), to_yocto("10")),
        E303_EXCEED_FARM_NUM_IN_SEED
    );

    // 4 : E203_EXCEED_FARM_NUM_IN_BOOST
    let mut affected_seeds = HashMap::new();
    for i in 0..MAX_NUM_SEEDS_PER_BOOSTER {
        let temp_seed_id = format!("{}{}", seed_id, i);
        e.create_seed(&e.owner, &temp_seed_id, TOKEN_DECIMALS as u32, None, None).assert_success();
        for _ in 0..e.get_config().max_num_farms_per_booster / MAX_NUM_SEEDS_PER_BOOSTER as u32 {
            e.create_farm(&e.owner, &temp_seed_id, &tokens.nref, to_sec(e.current_time()), to_yocto("10")).assert_success();
            affected_seeds.insert(temp_seed_id.clone(), 100);
        }
    }
    let booster_id = tokens.love_ref.account_id.to_string();
    let booster_info = BoosterInfo { booster_decimal: 100, affected_seeds: affected_seeds.clone()};
    e.create_seed(&e.owner, &booster_id, TOKEN_DECIMALS as u32, None, None).assert_success();
    assert_seed(e.get_seed(&booster_id), &booster_id, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);

    e.modify_booster(&e.owner, &booster_id, &booster_info).assert_success();

    assert_err!(print e.create_farm(&e.owner, &format!("{}{}", seed_id, 0), &tokens.nref, to_sec(start_at), to_yocto("10")));
    assert_err!(
        e.create_farm(&e.owner, &format!("{}{}", seed_id, 0), &tokens.nref, to_sec(start_at), to_yocto("10")),
        E203_EXCEED_FARM_NUM_IN_BOOST
    );
}

#[test]
fn test_cancle_farm() {
    let e = init_env();
    let users = Users::init(&e);
    let tokens = Tokens::init(&e);

    let inner_id = "0".to_string();
    let seed_id = e.mft_seed_id(&inner_id);
    let farm_id = format!("{}#{}", seed_id, 0);

    e.create_seed(&e.owner, &seed_id, TOKEN_DECIMALS as u32, None, None).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);

    let start_at = e.current_time();
    e.create_farm(&e.owner, &seed_id, &tokens.nref, to_sec(start_at), to_yocto("10")).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 1, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);

    assert_farm_info(e.get_farm(&farm_id), &farm_id, &tokens.nref, to_sec(start_at), to_yocto("10"));
    assert_farm_detail(e.get_farm(&farm_id), 0, start_at, 0, 0, 0, 0, Some(FarmStatus::Created));

    // success
    e.skip_time(to_sec(NANOS_PER_DAY));
    assert_eq!(e.get_metadata().farm_count.0, 1);
    e.cancel_farm(&e.owner, &farm_id).assert_success();
    assert_eq!(e.get_metadata().farm_count.0, 0);

    
    let farm_id = format!("{}#{}", seed_id, 1);
    let start_at = e.current_time();
    e.create_farm(&e.owner, &seed_id, &tokens.nref, to_sec(start_at), to_yocto("10")).assert_success();
    
    // error scene
    // 1 : E002_NOT_ALLOWED
    assert_err!(
        e.cancel_farm(&users.alice, &farm_id),
        E002_NOT_ALLOWED
    );

    // 2 : E406_INVALID_FARM_ID
    assert_err!(
        e.cancel_farm(&e.owner, &"farm_id".to_string()),
        E406_INVALID_FARM_ID
    );

    // 3 :  E301_SEED_NOT_EXIST
    assert_err!(
        e.cancel_farm(&e.owner, &format!("{}#{}", "seed_id", 1)),
        E301_SEED_NOT_EXIST
    );

    // 4 : E401_FARM_NOT_EXIST
    assert_err!(
        e.cancel_farm(&e.owner, &format!("{}#{}", seed_id, 2)),
        E401_FARM_NOT_EXIST
    );

    // 5 : E403_FARM_ALREADY_DEPOSIT_REWARD
    e.ft_mint(&tokens.nref, &users.operator, to_yocto("10000"));
    assert_eq!(e.ft_balance_of(&tokens.nref, &users.operator), to_yocto("10000"));
    e.deposit_reward(&tokens.nref, &users.operator, to_yocto("100"), &farm_id).assert_success();
    assert_farm_detail(e.get_farm(&farm_id), to_yocto("100"), start_at, 0, 0, 0, 0, Some(FarmStatus::Created));
    assert_err!(
        e.cancel_farm(&e.owner, &farm_id),
        E403_FARM_ALREADY_DEPOSIT_REWARD
    );
    assert_eq!(e.get_metadata().farm_count.0, 1);
}

#[test]
fn test_remove_farm_from_seed(){
    let e = init_env();
    let users = Users::init(&e);
    let tokens = Tokens::init(&e);

    let inner_id = "0".to_string();
    let seed_id = e.mft_seed_id(&inner_id);
    let farm_id = format!("{}#{}", seed_id, 0);

    println!("> create_seed at : {}", e.current_time());
    e.create_seed(&e.owner, &seed_id, TOKEN_DECIMALS as u32, None, None).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);

    let start_at = e.current_time();
    println!("> create_farm at : {}", e.current_time());
    e.create_farm(&e.owner, &seed_id, &tokens.nref, to_sec(start_at), to_yocto("10")).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 1, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    assert_farm_info(e.get_farm(&farm_id), &farm_id, &tokens.nref, to_sec(start_at), to_yocto("10"));
    assert_farm_detail(e.get_farm(&farm_id), 0, start_at, 0, 0, 0, 0, Some(FarmStatus::Created));


    println!("> deposit_reward at : {}", e.current_time());
    e.ft_mint(&tokens.nref, &users.operator, to_yocto("10000"));
    assert_eq!(e.ft_balance_of(&tokens.nref, &users.operator), to_yocto("10000"));
    e.deposit_reward(&tokens.nref, &users.operator, to_yocto("10"), &farm_id).assert_success();
    assert_farm_detail(e.get_farm(&farm_id), to_yocto("10"), start_at, 0, 0, 0, 0, Some(FarmStatus::Created));

    // error scene
    // 1 : E002_NOT_ALLOWED
    assert_err!(
        e.remove_farm_from_seed(&users.alice, &farm_id),
        E002_NOT_ALLOWED
    );

    // 2 : E406_INVALID_FARM_ID
    assert_err!(
        e.remove_farm_from_seed(&e.owner, &"farm_id".to_string()),
        E406_INVALID_FARM_ID
    );

    // 3 :  E301_SEED_NOT_EXIST
    assert_err!(
        e.remove_farm_from_seed(&e.owner, &format!("{}#{}", "seed_id", 1)),
        E301_SEED_NOT_EXIST
    );

    // 4 : E401_FARM_NOT_EXIST
    assert_err!(
        e.remove_farm_from_seed(&e.owner, &format!("{}#{}", seed_id, 2)),
        E401_FARM_NOT_EXIST
    );

    // 5 : E405_FARM_NOT_ENDED
    assert_err!(
        e.remove_farm_from_seed(&e.owner, &farm_id),
        E405_FARM_NOT_ENDED
    );

    // success
    e.skip_time(to_sec(NANOS_PER_DAY));
    assert_eq!(e.get_metadata().farm_count.0, 1);
    assert_eq!(e.get_metadata().outdated_farm_count.0, 0);
    e.remove_farm_from_seed(&e.owner, &farm_id).assert_success();
    assert_eq!(e.get_metadata().farm_count.0, 0);
    assert_eq!(e.get_metadata().outdated_farm_count.0, 1);
}