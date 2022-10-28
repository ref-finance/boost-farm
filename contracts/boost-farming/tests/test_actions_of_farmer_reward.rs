mod setup;
use crate::setup::*;

#[test]
fn test_claim_reward_by_seed(){
    let e = init_env();
    let users = Users::init(&e);
    let tokens = Tokens::init(&e);

    let inner_id = "0".to_string();
    let token_id = format!(":{}", inner_id);
    let seed_id = e.mft_seed_id(&inner_id);
    let farm_id = format!("{}#{}", seed_id, 0);

    println!("> create_seed at : {}", e.current_time());
    e.create_seed(&e.owner, &seed_id, TOKEN_DECIMALS as u32, None, None).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);

    println!("> farmer1 mint mft at : {}", e.current_time());
    e.mft_mint(&inner_id, &users.farmer1, to_yocto("100"));
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id), to_yocto("100"));
    e.storage_deposit_self_to_farming(&users.farmer1).assert_success();

    println!("> farming register mft at : {}", e.current_time());
    e.mft_storage_deposit(&token_id, &e.farming_contract.user_account);

    let start_at = e.current_time();
    println!("> create_farm at : {}", e.current_time());
    e.create_farm(&e.owner, &seed_id, &tokens.nref, to_sec(start_at), to_yocto("10")).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 1, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    assert_farm_info(e.get_farm(&farm_id), &farm_id, &tokens.nref, to_sec(start_at), to_yocto("10"));
    assert_farm_detail(e.get_farm(&farm_id), 0, start_at, 0, 0, 0, 0, Some(FarmStatus::Created));

    println!("> deposit_reward at : {}", e.current_time());
    e.ft_mint(&tokens.nref, &users.operator, to_yocto("10000"));
    assert_eq!(e.ft_balance_of(&tokens.nref, &users.operator), to_yocto("10000"));
    e.deposit_reward(&tokens.nref, &users.operator, to_yocto("100"), &farm_id).assert_success();
    assert_farm_detail(e.get_farm(&farm_id), to_yocto("100"), start_at, 0, 0, 0, 0, Some(FarmStatus::Created));

    println!("> farmer1 mft_stake_lock_seed at : {}", e.current_time());
    e.mft_stake_lock_seed(&token_id, &users.farmer1, to_yocto("50"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id), 0, to_yocto("50"), to_yocto("100"), e.current_time() + to_nano(DEFAULT_MAX_LOCKING_DURATION_SEC), DEFAULT_MAX_LOCKING_DURATION_SEC);

    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id), to_yocto("100"), e.current_time(), to_yocto("10"), 0, 0, 0, Some(FarmStatus::Running));

    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref), to_yocto("10"));

    // error scene 
    // 1 : E100_ACC_NOT_REGISTERED
    assert_err!(
        e.claim_reward_by_seed(&users.farmer2, &seed_id),
        E100_ACC_NOT_REGISTERED
    );

    // error scene 
    // 2 : E301_SEED_NOT_EXIST
    assert_err!(
        e.claim_reward_by_seed(&users.farmer1, &format!("{}#{}", "seed_id", 1)),
        E301_SEED_NOT_EXIST
    );

    assert_eq!(e.get_farmer_reward(&users.farmer1, &tokens.nref), 0);
    e.claim_reward_by_seed(&users.farmer1, &seed_id).assert_success();
    assert_eq!(e.get_farmer_reward(&users.farmer1, &tokens.nref), to_yocto("10"));
}


#[test]
fn test_withdraw_reward(){
    let e = init_env();
    let users = Users::init(&e);
    let tokens = Tokens::init(&e);

    let inner_id = "0".to_string();
    let token_id = format!(":{}", inner_id);
    let seed_id = e.mft_seed_id(&inner_id);
    let farm_id = format!("{}#{}", seed_id, 0);

    println!("> create_seed at : {}", e.current_time());
    e.create_seed(&e.owner, &seed_id, TOKEN_DECIMALS as u32, None, None).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);

    println!("> farmer1 mint mft at : {}", e.current_time());
    e.mft_mint(&inner_id, &users.farmer1, to_yocto("100"));
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id), to_yocto("100"));
    e.storage_deposit_self_to_farming(&users.farmer1).assert_success();

    println!("> farming register mft at : {}", e.current_time());
    e.mft_storage_deposit(&token_id, &e.farming_contract.user_account);

    let start_at = e.current_time();
    println!("> create_farm at : {}", e.current_time());
    e.create_farm(&e.owner, &seed_id, &tokens.nref, to_sec(start_at), to_yocto("10")).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 1, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    assert_farm_info(e.get_farm(&farm_id), &farm_id, &tokens.nref, to_sec(start_at), to_yocto("10"));
    assert_farm_detail(e.get_farm(&farm_id), 0, start_at, 0, 0, 0, 0, Some(FarmStatus::Created));

    println!("> deposit_reward at : {}", e.current_time());
    e.ft_mint(&tokens.nref, &users.operator, to_yocto("10000"));
    assert_eq!(e.ft_balance_of(&tokens.nref, &users.operator), to_yocto("10000"));
    e.deposit_reward(&tokens.nref, &users.operator, to_yocto("100"), &farm_id).assert_success();
    assert_farm_detail(e.get_farm(&farm_id), to_yocto("100"), start_at, 0, 0, 0, 0, Some(FarmStatus::Created));

    println!("> farmer1 mft_stake_lock_seed at : {}", e.current_time());
    e.mft_stake_lock_seed(&token_id, &users.farmer1, to_yocto("50"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id), 0, to_yocto("50"), to_yocto("100"), e.current_time() + to_nano(DEFAULT_MAX_LOCKING_DURATION_SEC), DEFAULT_MAX_LOCKING_DURATION_SEC);

    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id), to_yocto("100"), e.current_time(), to_yocto("10"), 0, 0, 0, Some(FarmStatus::Running));

    assert_eq!(e.get_farmer_reward(&users.farmer1, &tokens.nref), 0);
    e.claim_reward_by_seed(&users.farmer1, &seed_id).assert_success();
    assert_eq!(e.get_farmer_reward(&users.farmer1, &tokens.nref), to_yocto("10"));

    // error scene 
    // 1 : E100_ACC_NOT_REGISTERED
    assert_err!(
        e.withdraw_reward(&users.farmer2, &tokens.nref, None),
        E100_ACC_NOT_REGISTERED
    );

    // 2 : The account farmer1.near is not registered
    let outcome = e.withdraw_reward(&users.farmer1, &tokens.nref, None);
    assert_eq!(false, outcome.unwrap_json::<bool>());
    assert_err!(
        outcome,
        "The account farmer1.near is not registered"
    );

    e.ft_storage_deposit(&users.farmer1, &tokens.nref);
    assert_eq!(e.get_farmer_reward(&users.farmer1, &tokens.nref), to_yocto("10"));
    assert_eq!(true, e.withdraw_reward(&users.farmer1, &tokens.nref, None).unwrap_json::<bool>());
    assert_eq!(e.get_farmer_reward(&users.farmer1, &tokens.nref), 0);
    assert_eq!(e.ft_balance_of(&tokens.nref, &users.farmer1), to_yocto("10"));
}