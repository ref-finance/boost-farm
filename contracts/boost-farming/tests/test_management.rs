mod setup;
use crate::setup::*;

#[test]
fn test_modify_daily_reward(){
    let e = init_env();
    let users = Users::init(&e);
    let tokens = Tokens::init(&e);

    let inner_id = "0".to_string();
    let seed_id = e.mft_seed_id(&inner_id);
    let farm_id = format!("{}#{}", seed_id, 0);

    println!("> create_seed at : {}", e.current_time());
    e.create_seed(&e.owner, &seed_id, TOKEN_DECIMALS as u32, None, None).assert_success();
    println!("> create_farm at : {}", e.current_time());
    e.create_farm(&e.owner, &seed_id, &tokens.nref, to_sec(e.current_time()), to_yocto("10")).assert_success();

    // error scene 
    // 1 : Requires attached deposit of exactly 1 yoctoNEAR
    assert_err!(
        e.modify_daily_reward(&users.bob, &farm_id, to_yocto("2"), 0),
        "Requires attached deposit of exactly 1 yoctoNEAR"
    );

    // 2 : E002_NOT_ALLOWED  
    assert_err!(
        e.modify_daily_reward(&users.bob, &farm_id, to_yocto("2"), 1),
        E002_NOT_ALLOWED
    );

    e.extend_operators(&e.owner, vec![&users.bob], 1).assert_success();

    // 3 : E406_INVALID_FARM_ID
    assert_err!(
        e.modify_daily_reward(&users.bob, &"farm_id".to_string(), to_yocto("2"), 1),
        E406_INVALID_FARM_ID
    );

    // 4 : E301_SEED_NOT_EXIST
    assert_err!(
        e.modify_daily_reward(&users.bob, &format!("{}#{}", "seed_id", 0), to_yocto("2"), 1),
        E301_SEED_NOT_EXIST
    );

    // 5 : E401_FARM_NOT_EXIST
    assert_err!(
        e.modify_daily_reward(&users.bob, &format!("{}#{}", seed_id, 1), to_yocto("2"), 1),
        E401_FARM_NOT_EXIST
    );

    // success
    assert_farm_info(e.get_farm(&farm_id), &farm_id, &tokens.nref, to_sec(e.current_time()), to_yocto("10"));
    e.modify_daily_reward(&users.bob, &farm_id, to_yocto("2"), 1).assert_success();
    assert_farm_info(e.get_farm(&farm_id), &farm_id, &tokens.nref, to_sec(e.current_time()), to_yocto("2"));
}

#[test]
fn test_modify_locking_policy(){
    let e = init_env();
    let users = Users::init(&e);
    
    // error scene 
    // 1 : Requires attached deposit of exactly 1 yoctoNEAR
    assert_err!(
        e.modify_locking_policy(&users.bob, 1000, 100, 0),
        "Requires attached deposit of exactly 1 yoctoNEAR"
    );

    // 2 : E002_NOT_ALLOWED  
    assert_err!(
        e.modify_locking_policy(&users.bob, 1000, 100, 1),
        E002_NOT_ALLOWED
    );

    e.extend_operators(&e.owner, vec![&users.bob], 1).assert_success();

    // 3 : E200_INVALID_RATIO
    assert_err!(
        e.modify_locking_policy(&users.bob, 1000, 100, 1),
        E200_INVALID_RATIO
    );

    // success
    assert_eq!(e.get_config().maximum_locking_duration_sec, DEFAULT_MAX_LOCKING_DURATION_SEC);
    assert_eq!(e.get_config().max_locking_multiplier, DEFAULT_MAX_LOCKING_REWARD_RATIO);
    e.modify_locking_policy(&users.bob,  1000,12000, 1).assert_success();
    assert_eq!(e.get_config().maximum_locking_duration_sec, 1000);
    assert_eq!(e.get_config().max_locking_multiplier, 12000);
}

#[test]
fn test_modify_max_farm_num_per_seed(){
    let e = init_env();
    let users = Users::init(&e);
    
    // error scene 
    // 1 : Requires attached deposit of exactly 1 yoctoNEAR
    assert_err!(
        e.modify_max_farm_num_per_seed(&users.bob, 100, 0),
        "Requires attached deposit of exactly 1 yoctoNEAR"
    );

    // 2 : E002_NOT_ALLOWED  
    assert_err!(
        e.modify_max_farm_num_per_seed(&users.bob, 100, 1),
        E002_NOT_ALLOWED
    );

    e.extend_operators(&e.owner, vec![&users.bob], 1).assert_success();

    // success
    assert_eq!(e.get_config().max_num_farms_per_seed, DEFAULT_MAX_NUM_FARMS_PER_SEED);
    e.modify_max_farm_num_per_seed(&users.bob, 20, 1).assert_success();
    assert_eq!(e.get_config().max_num_farms_per_seed, 20);
}

#[test]
fn test_modify_default_slash_rate(){
    let e = init_env();
    let users = Users::init(&e);
    
    // error scene 
    // 1 : Requires attached deposit of exactly 1 yoctoNEAR
    assert_err!(
        e.modify_default_slash_rate(&users.bob, 100, 0),
        "Requires attached deposit of exactly 1 yoctoNEAR"
    );

    // 2 : E002_NOT_ALLOWED
    assert_err!(
        e.modify_default_slash_rate(&users.bob, 100, 1),
        E002_NOT_ALLOWED
    );

    // 3 : E205_INVALID_SLASH_RATE
    assert_err!(
        e.modify_default_slash_rate(&e.owner, 100000, 1),
        E205_INVALID_SLASH_RATE
    );

    e.extend_operators(&e.owner, vec![&users.bob], 1).assert_success();

    // success
    assert_eq!(e.get_config().seed_slash_rate, DEFAULT_SEED_SLASH_RATE);
    e.modify_default_slash_rate(&users.bob, 20, 1).assert_success();
    assert_eq!(e.get_config().seed_slash_rate, 20);
}

#[test]
fn test_modify_seed_min_deposit(){
    let e = init_env();
    let users = Users::init(&e);

    let inner_id = "0".to_string();
    let token_id = format!(":{}", inner_id);
    let seed_id = e.mft_seed_id(&inner_id);

    println!("> create_seed at : {}", e.current_time());
    e.create_seed(&e.owner, &seed_id, TOKEN_DECIMALS as u32, None, None).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);

    // error scene 
    // 1 : Requires attached deposit of exactly 1 yoctoNEAR
    assert_err!(
        e.modify_seed_min_deposit(&users.bob, &seed_id, 100, 0),
        "Requires attached deposit of exactly 1 yoctoNEAR"
    );

    // 2 : E002_NOT_ALLOWED  
    assert_err!(
        e.modify_seed_min_deposit(&users.bob, &seed_id, 100, 1),
        E002_NOT_ALLOWED
    );

    // 3 : E301_SEED_NOT_EXIST  
    assert_err!(
        e.modify_seed_min_deposit(&e.owner, &"seed_id".to_string(), 100, 1),
        E301_SEED_NOT_EXIST
    );

    e.extend_operators(&e.owner, vec![&users.bob], 1).assert_success();

    e.mft_mint(&inner_id, &users.farmer1, to_yocto("100"));
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id), to_yocto("100"));
    e.storage_deposit_self_to_farming(&users.farmer1).assert_success();

    // 4 : mft_stake_free_seed with E307_BELOW_MIN_DEPOSIT
    e.mft_storage_deposit(&token_id, &e.farming_contract.user_account);
    assert_err!(e.mft_stake_free_seed(&users.farmer1, &token_id, 50), E307_BELOW_MIN_DEPOSIT);

    // 5 : mft_stake_lock_seed with E307_BELOW_MIN_DEPOSIT
    assert_err!(e.mft_stake_lock_seed(&token_id, &users.farmer1, 50, DEFAULT_MAX_LOCKING_DURATION_SEC), E307_BELOW_MIN_DEPOSIT);

    // success
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    e.modify_seed_min_deposit(&users.bob, &seed_id, 20, 1).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 0, 0, 0, 20, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    e.mft_stake_free_seed(&users.farmer1, &token_id, 50).assert_success();
    e.mft_stake_lock_seed(&token_id, &users.farmer1, 50, DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
}

#[test]
fn test_modify_seed_min_locking_duration(){
    let e = init_env();
    let users = Users::init(&e);

    let inner_id = "0".to_string();
    let seed_id = e.mft_seed_id(&inner_id);

    println!("> create_seed at : {}", e.current_time());
    e.create_seed(&e.owner, &seed_id, TOKEN_DECIMALS as u32, None, None).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);

    // error scene 
    // 1 : Requires attached deposit of exactly 1 yoctoNEAR
    assert_err!(
        e.modify_seed_min_locking_duration(&users.bob, &seed_id, 100, 0),
        "Requires attached deposit of exactly 1 yoctoNEAR"
    );

    // 2 : E002_NOT_ALLOWED
    assert_err!(
        e.modify_seed_min_locking_duration(&users.bob, &seed_id, 100, 1),
        E002_NOT_ALLOWED
    );

    e.extend_operators(&e.owner, vec![&users.bob], 1).assert_success();

    // 3 : E201_INVALID_DURATION
    assert_err!(
        e.modify_seed_min_locking_duration(&users.bob, &seed_id, DEFAULT_MAX_LOCKING_DURATION_SEC + 1, 1),
        E201_INVALID_DURATION
    );

    // 4 : E301_SEED_NOT_EXIST
    assert_err!(
        e.modify_seed_min_locking_duration(&users.bob, &"seed_id".to_string(), 100, 1),
        E301_SEED_NOT_EXIST
    );

    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    e.modify_seed_min_locking_duration(&users.bob, &seed_id, 100, 1).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, 100);
}

#[test]
fn test_modify_seed_slash_rate(){
    let e = init_env();
    let users = Users::init(&e);

    let inner_id = "0".to_string();
    let seed_id = e.mft_seed_id(&inner_id);

    println!("> create_seed at : {}", e.current_time());
    e.create_seed(&e.owner, &seed_id, TOKEN_DECIMALS as u32, None, None).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);

    // error scene 
    // 1 : Requires attached deposit of exactly 1 yoctoNEAR
    assert_err!(
        e.modify_seed_slash_rate(&users.bob, &seed_id, 100, 0),
        "Requires attached deposit of exactly 1 yoctoNEAR"
    );

    // 2 : E002_NOT_ALLOWED  
    assert_err!(
        e.modify_seed_slash_rate(&users.bob, &seed_id, 100, 1),
        E002_NOT_ALLOWED
    );

    // 3 : E301_SEED_NOT_EXIST  
    assert_err!(
        e.modify_seed_slash_rate(&e.owner, &"seed_id".to_string(), 100, 1),
        E301_SEED_NOT_EXIST
    );

    e.extend_operators(&e.owner, vec![&users.bob], 1).assert_success();

    // success
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    e.modify_seed_slash_rate(&users.bob, &seed_id, 20, 1).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, 20, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
}

#[test]
fn test_withdraw_seed_slashed(){
    let e = init_env();
    let users = Users::init(&e);

    let inner_id = "0".to_string();
    let token_id = format!(":{}", inner_id);
    let seed_id = e.mft_seed_id(&inner_id);

    println!("> create_seed at : {}", e.current_time());
    e.create_seed(&e.owner, &seed_id, TOKEN_DECIMALS as u32, None, None).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);

    println!("> farmer1 mint mft at : {}", e.current_time());
    e.mft_mint(&inner_id, &users.farmer1, to_yocto("100"));
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id), to_yocto("100"));
    e.storage_deposit_self_to_farming(&users.farmer1).assert_success();

    println!("> farming register mft at : {}", e.current_time());
    e.mft_storage_deposit(&token_id, &e.farming_contract.user_account);

    println!("> farmer1 mft_stake_lock_seed at : {}", e.current_time());
    e.mft_stake_lock_seed(&token_id, &users.farmer1, to_yocto("50"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    let farmer1_unlock_time = e.current_time() + to_nano(DEFAULT_MAX_LOCKING_DURATION_SEC);
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id), 0, to_yocto("50"), to_yocto("100"), farmer1_unlock_time, DEFAULT_MAX_LOCKING_DURATION_SEC);


    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());

    // error scene 
    // 1 : Requires attached deposit of exactly 1 yoctoNEAR
    assert_err!(
        e.withdraw_seed_slashed(&e.owner, &seed_id, 0),
        "Requires attached deposit of exactly 1 yoctoNEAR"
    );

    // 2 : E002_NOT_ALLOWED
    assert_err!(
        e.withdraw_seed_slashed(&users.bob, &seed_id, 1),
        E002_NOT_ALLOWED
    );

    // 3 : E101_INSUFFICIENT_BALANCE
    assert_err!(
        e.withdraw_seed_slashed(&e.owner, &seed_id, 1),
        E101_INSUFFICIENT_BALANCE
    );

    println!("> farmer1 force_unlock all seed at : {}, have slashed", e.current_time());
    e.force_unlock(&users.farmer1, &seed_id, to_yocto("50")).assert_success();
    let full_slashed = u128_ratio(to_yocto("50"), DEFAULT_SEED_SLASH_RATE as u128, BP_DENOM);
    let seed_slashed = u128_ratio(full_slashed, (farmer1_unlock_time - e.current_time()) as u128, to_nano(DEFAULT_MAX_LOCKING_DURATION_SEC) as u128);
    let free_amount = to_yocto("50") - seed_slashed;
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id), free_amount, 0, 0, 0, 0);


    // 3 : ERR_RECEIVER_NOT_REGISTERED
    assert_err!(
        e.withdraw_seed_slashed(&e.owner, &seed_id, 1),
        "ERR_RECEIVER_NOT_REGISTERED"
    );

    // success
    println!("> owner register mft at : {}", e.current_time());
    e.mft_storage_deposit(&token_id, &e.owner);
    assert_eq!(e.mft_balance_of(&e.owner, &token_id), 0);
    e.withdraw_seed_slashed(&e.owner, &seed_id, 1).assert_success();
    assert_eq!(e.mft_balance_of(&e.owner, &token_id), seed_slashed);
}

#[test]
fn test_return_seed_lostfound(){
    let e = init_env();
    let users = Users::init(&e);

    let inner_id = "0".to_string();
    let token_id = format!(":{}", inner_id);
    let seed_id = e.mft_seed_id(&inner_id);

    println!("> create_seed at : {}", e.current_time());
    e.create_seed(&e.owner, &seed_id, TOKEN_DECIMALS as u32, None, None).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);

    println!("> farmer1 mint mft at : {}", e.current_time());
    e.mft_mint(&inner_id, &users.farmer1, to_yocto("100"));
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id), to_yocto("100"));
    e.storage_deposit_self_to_farming(&users.farmer1).assert_success();

    println!("> farming register mft at : {}", e.current_time());
    e.mft_storage_deposit(&token_id, &e.farming_contract.user_account);

    println!("> farmer1 mft_stake_free_seed at : {}", e.current_time());
    e.mft_stake_free_seed(&users.farmer1, &token_id, to_yocto("100")).assert_success();
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id), to_yocto("100"), 0, 0, 0, 0);

    // error scene 
    // 1 : Requires attached deposit of exactly 1 yoctoNEAR
    assert_err!(
        e.return_seed_lostfound(&e.owner, &users.farmer1, &seed_id, to_yocto("100"), 0),
        "Requires attached deposit of exactly 1 yoctoNEAR"
    );
    
    // 2 : E002_NOT_ALLOWED
    assert_err!(
        e.return_seed_lostfound(&users.bob, &users.farmer1, &seed_id, to_yocto("100"), 1),
        E002_NOT_ALLOWED
    );

    // 3 : E101_INSUFFICIENT_BALANCE
    assert_err!(
        e.return_seed_lostfound(&e.owner, &users.farmer1, &seed_id, to_yocto("100"), 1),
        E101_INSUFFICIENT_BALANCE
    );

    // 4 : E100_ACC_NOT_REGISTERED
    assert_err!(
        e.return_seed_lostfound(&e.owner, &users.dude, &seed_id, to_yocto("100"), 1),
        E100_ACC_NOT_REGISTERED
    );

    e.mft_unregister(&token_id, &users.farmer1);
    let outcome = e.unlock_and_withdraw_seed(&users.farmer1, &seed_id, 0, to_yocto("100"));
    assert_eq!(false, outcome.unwrap_json::<bool>());
    assert_err!(
        outcome,
        "ERR_RECEIVER_NOT_REGISTERED"
    );
    assert_eq!(e.list_lostfound().get(&seed_id).unwrap().0, to_yocto("100"));
    
    // 5 : ERR_RECEIVER_NOT_REGISTERED
    assert_err!(
        e.return_seed_lostfound(&e.owner, &users.farmer1, &seed_id, to_yocto("100"), 1),
        "ERR_RECEIVER_NOT_REGISTERED"
    );
    
    // success
    e.mft_storage_deposit(&token_id, &users.farmer1, );
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id), 0);
    e.return_seed_lostfound(&e.owner, &users.farmer1, &seed_id, to_yocto("100"), 1).assert_success();
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id), to_yocto("100"));
}

#[test]
fn test_withdraw_from_beneficiary_account(){
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

    // error scene 
    // 1 : Requires attached deposit of exactly 1 yoctoNEAR
    assert_err!(
        e.withdraw_from_beneficiary_account(&users.bob, &farm_id, 0),
        "Requires attached deposit of exactly 1 yoctoNEAR"
    );

    // 2 : E002_NOT_ALLOWED  
    assert_err!(
        e.withdraw_from_beneficiary_account(&users.bob, &farm_id,  1),
        E002_NOT_ALLOWED
    );

    e.extend_operators(&e.owner, vec![&users.bob], 1).assert_success();

    // 3 : E401_FARM_NOT_EXIST
    assert_err!(
        e.withdraw_from_beneficiary_account(&users.bob, &format!("{}#{}", seed_id, 1), 1),
        E401_FARM_NOT_EXIST
    );

    println!("> farmer1 mint mft at : {}", e.current_time());
    e.mft_mint(&inner_id, &users.farmer1, to_yocto("100"));
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id), to_yocto("100"));

    println!("> farmer1 register farming at : {}", e.current_time());
    e.storage_deposit_self_to_farming(&users.farmer1).assert_success();
    assert_eq!(e.get_metadata().farmer_count.0, 1);

    println!("> farmer2 mint mft at : {}", e.current_time());
    e.mft_mint(&inner_id, &users.farmer2, to_yocto("100"));
    assert_eq!(e.mft_balance_of(&users.farmer2, &token_id), to_yocto("100"));

    println!("> farmer2 register farming at : {}", e.current_time());
    e.storage_deposit_self_to_farming(&users.farmer2).assert_success();
    assert_eq!(e.get_metadata().farmer_count.0, 2);

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

    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id), to_yocto("100"), e.current_time(), to_yocto("10"), 0, to_yocto("10"), 0, Some(FarmStatus::Running));

    // test if receiver account doesn't register the token
    assert_eq!(e.ft_balance_of(&tokens.nref,&e.owner), 0);
    e.withdraw_from_beneficiary_account(&users.bob, &farm_id,  1);
    assert_eq!(e.ft_balance_of(&tokens.nref,&e.owner), 0);

    assert_farm_detail(e.get_farm(&farm_id), to_yocto("100"), e.current_time(), to_yocto("10"), 0, to_yocto("10"), 0, Some(FarmStatus::Running));

    //
    e.ft_storage_deposit(&e.owner,&tokens.nref);
    assert_eq!(e.ft_balance_of(&tokens.nref,&e.owner), 0);
    e.withdraw_from_beneficiary_account(&users.bob, &farm_id,  1);
    assert_eq!(e.ft_balance_of(&tokens.nref,&e.owner), to_yocto("10"));

    assert_farm_detail(e.get_farm(&farm_id), to_yocto("100"), e.current_time(), to_yocto("10"), 0, to_yocto("10"), to_yocto("10"), Some(FarmStatus::Running));

    e.skip_time(to_sec(NANOS_PER_DAY * 11));
    e.remove_farm_from_seed(&e.owner, &farm_id).assert_success();
    assert_eq!(e.ft_balance_of(&tokens.nref,&e.owner), to_yocto("10"));
    assert_eq!(e.get_outdated_farm(&farm_id).amount_of_withdrew_beneficiary, to_yocto("10"));
    e.withdraw_from_outdated_beneficiary_account(&users.bob, &farm_id,  1);
    assert_eq!(e.ft_balance_of(&tokens.nref,&e.owner), to_yocto("100"));
    assert_eq!(e.get_outdated_farm(&farm_id).amount_of_withdrew_beneficiary, to_yocto("100"));
}

#[test]
fn test_withdraw_from_outdated_beneficiary_account(){
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

    // error scene 
    // 1 : Requires attached deposit of exactly 1 yoctoNEAR
    assert_err!(
        e.withdraw_from_outdated_beneficiary_account(&users.bob, &farm_id, 0),
        "Requires attached deposit of exactly 1 yoctoNEAR"
    );

    // 2 : E002_NOT_ALLOWED  
    assert_err!(
        e.withdraw_from_outdated_beneficiary_account(&users.bob, &farm_id,  1),
        E002_NOT_ALLOWED
    );

    e.extend_operators(&e.owner, vec![&users.bob], 1).assert_success();

    // 3 : E401_FARM_NOT_EXIST
    assert_err!(
        e.withdraw_from_outdated_beneficiary_account(&users.bob, &format!("{}#{}", seed_id, 1), 1),
        E401_FARM_NOT_EXIST
    );

    println!("> farmer1 mint mft at : {}", e.current_time());
    e.mft_mint(&inner_id, &users.farmer1, to_yocto("100"));
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id), to_yocto("100"));

    println!("> farmer1 register farming at : {}", e.current_time());
    e.storage_deposit_self_to_farming(&users.farmer1).assert_success();
    assert_eq!(e.get_metadata().farmer_count.0, 1);

    println!("> farmer2 mint mft at : {}", e.current_time());
    e.mft_mint(&inner_id, &users.farmer2, to_yocto("100"));
    assert_eq!(e.mft_balance_of(&users.farmer2, &token_id), to_yocto("100"));

    println!("> farmer2 register farming at : {}", e.current_time());
    e.storage_deposit_self_to_farming(&users.farmer2).assert_success();
    assert_eq!(e.get_metadata().farmer_count.0, 2);

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

    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id), to_yocto("100"), e.current_time(), to_yocto("10"), 0, to_yocto("10"), 0, Some(FarmStatus::Running));

    e.skip_time(to_sec(NANOS_PER_DAY * 11));
    e.remove_farm_from_seed(&e.owner, &farm_id).assert_success();

    // test if receiver account doesn't register the token
    assert_err!(e.withdraw_from_outdated_beneficiary_account(&users.bob, &farm_id,  1), 
        "The account owner.near is not registered");

    assert_eq!(e.ft_balance_of(&tokens.nref,&e.owner), to_yocto("0"));
    assert_eq!(e.get_outdated_farm(&farm_id).amount_of_withdrew_beneficiary, to_yocto("0"));
    e.ft_storage_deposit(&e.owner,&tokens.nref);
    e.withdraw_from_outdated_beneficiary_account(&users.bob, &farm_id,  1);
    assert_eq!(e.ft_balance_of(&tokens.nref,&e.owner), to_yocto("100"));
    assert_eq!(e.get_outdated_farm(&farm_id).amount_of_withdrew_beneficiary, to_yocto("100"));
}

#[test]
fn test_withdraw_from_undistributed_reward(){
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

    // error scene 
    // 1 : Requires attached deposit of exactly 1 yoctoNEAR
    assert_err!(
        e.withdraw_from_beneficiary_account(&users.bob, &farm_id, 0),
        "Requires attached deposit of exactly 1 yoctoNEAR"
    );

    // 2 : E002_NOT_ALLOWED  
    assert_err!(
        e.withdraw_from_beneficiary_account(&users.bob, &farm_id,  1),
        E002_NOT_ALLOWED
    );

    e.extend_operators(&e.owner, vec![&users.bob], 1).assert_success();

    // 3 : E401_FARM_NOT_EXIST
    assert_err!(
        e.withdraw_from_beneficiary_account(&users.bob, &format!("{}#{}", seed_id, 1), 1),
        E401_FARM_NOT_EXIST
    );

    println!("> farmer1 mint mft at : {}", e.current_time());
    e.mft_mint(&inner_id, &users.farmer1, to_yocto("100"));
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id), to_yocto("100"));

    println!("> farmer1 register farming at : {}", e.current_time());
    e.storage_deposit_self_to_farming(&users.farmer1).assert_success();
    assert_eq!(e.get_metadata().farmer_count.0, 1);

    println!("> farmer2 mint mft at : {}", e.current_time());
    e.mft_mint(&inner_id, &users.farmer2, to_yocto("100"));
    assert_eq!(e.mft_balance_of(&users.farmer2, &token_id), to_yocto("100"));

    println!("> farmer2 register farming at : {}", e.current_time());
    e.storage_deposit_self_to_farming(&users.farmer2).assert_success();
    assert_eq!(e.get_metadata().farmer_count.0, 2);

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

    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id), to_yocto("100"), e.current_time(), to_yocto("10"), 0, to_yocto("10"), 0, Some(FarmStatus::Running));

    // test if receiver account doesn't register the token
    assert_eq!(e.ft_balance_of(&tokens.nref,&e.owner), 0);
    e.withdraw_from_undistributed_reward(&users.bob, &farm_id, to_yocto("10"), 1);
    assert_eq!(e.ft_balance_of(&tokens.nref,&e.owner), 0);

    assert_farm_detail(e.get_farm(&farm_id), to_yocto("100"), e.current_time(), to_yocto("10"), 0, to_yocto("10"), 0, Some(FarmStatus::Running));    

    //
    e.ft_storage_deposit(&e.owner,&tokens.nref);
    assert_eq!(e.ft_balance_of(&tokens.nref,&e.owner), 0);
    println!("> withdraw part of undistributed reward");
    e.withdraw_from_undistributed_reward(&users.bob, &farm_id, to_yocto("10"), 1);
    assert_eq!(e.ft_balance_of(&tokens.nref,&e.owner), to_yocto("10"));

    assert_farm_detail(e.get_farm(&farm_id), to_yocto("90"), e.current_time(), to_yocto("10"), 0, to_yocto("10"), 0, Some(FarmStatus::Running));    
    println!("> withdraw the whole undistributed reward");
    e.withdraw_from_undistributed_reward(&users.bob, &farm_id, 0, 1);
    assert_eq!(e.ft_balance_of(&tokens.nref,&e.owner), to_yocto("90"));
    assert_farm_detail(e.get_farm(&farm_id), to_yocto("10"), e.current_time(), to_yocto("10"), 0, to_yocto("10"), 0, Some(FarmStatus::Ended));    
}