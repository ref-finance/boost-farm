mod setup;
use crate::setup::*;

#[test]
fn test_lock_free_seed(){
    let e = init_env();
    let users = Users::init(&e);

    let inner_id = "0".to_string();
    let token_id = format!(":{}", inner_id);
    let seed_id = e.mft_seed_id(&inner_id);

    let seed_id_cant_lock = e.mft_seed_id(&"1".to_string());

    println!("> create_seed at : {}", e.current_time());
    e.create_seed(&e.owner, &seed_id, TOKEN_DECIMALS as u32, None, None).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    e.create_seed(&e.owner, &seed_id_cant_lock, TOKEN_DECIMALS as u32, None, Some(0)).assert_success();
    assert_seed(e.get_seed(&seed_id_cant_lock), &seed_id_cant_lock, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, 0);

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
    // 1 : E100_ACC_NOT_REGISTERED
    assert_err!(
        e.lock_free_seed(&users.farmer2, &format!("{}#{}", "seed_id", 1), DEFAULT_MAX_LOCKING_DURATION_SEC, to_yocto("50")),
        E100_ACC_NOT_REGISTERED
    );

    // 2 : E301_SEED_NOT_EXIST
    assert_err!(
        e.lock_free_seed(&users.farmer1, &format!("{}#{}", "seed_id", 1), DEFAULT_MAX_LOCKING_DURATION_SEC, to_yocto("50")),
        E301_SEED_NOT_EXIST
    );

    // 3 : E300_FORBID_LOCKING
    assert_err!(
        e.lock_free_seed(&users.farmer1, &seed_id_cant_lock, DEFAULT_MAX_LOCKING_DURATION_SEC, to_yocto("50")),
        E300_FORBID_LOCKING
    );

    // 4 : E201_INVALID_DURATION
    assert_err!(
        e.lock_free_seed(&users.farmer1, &seed_id, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC - 1, to_yocto("50")),
        E201_INVALID_DURATION
    );

    // 5 : E201_INVALID_DURATION
    assert_err!(
        e.lock_free_seed(&users.farmer1, &seed_id, DEFAULT_MAX_LOCKING_DURATION_SEC + 1, to_yocto("50")),
        E201_INVALID_DURATION
    );

    // 6 : E101_INSUFFICIENT_BALANCE
    assert_err!(
        e.lock_free_seed(&users.farmer1, &seed_id, DEFAULT_MAX_LOCKING_DURATION_SEC, to_yocto("150")),
        E101_INSUFFICIENT_BALANCE
    );

    // success
    let farmer1_unlock_time = e.current_time() + to_nano(DEFAULT_MAX_LOCKING_DURATION_SEC);
    e.lock_free_seed(&users.farmer1, &seed_id, DEFAULT_MAX_LOCKING_DURATION_SEC, to_yocto("50")).assert_success();
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id), to_yocto("50"), to_yocto("50"), to_yocto("100"), farmer1_unlock_time, DEFAULT_MAX_LOCKING_DURATION_SEC);

    // 5 : E304_CAUSE_PRE_UNLOCK
    assert_err!(
        e.lock_free_seed(&users.farmer1, &seed_id, DEFAULT_MAX_LOCKING_DURATION_SEC - 1, to_yocto("50")),
        E304_CAUSE_PRE_UNLOCK
    );

    // append success
    e.lock_free_seed(&users.farmer1, &seed_id, DEFAULT_MAX_LOCKING_DURATION_SEC, to_yocto("50")).assert_success();
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id), 0, to_yocto("100"), to_yocto("200"), farmer1_unlock_time, DEFAULT_MAX_LOCKING_DURATION_SEC);
}

#[test]
fn test_unlock_and_unstake_seed(){
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
    e.mft_stake_lock_seed(&token_id, &users.farmer1, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id), 0, to_yocto("100"), to_yocto("200"), e.current_time() + to_nano(DEFAULT_MAX_LOCKING_DURATION_SEC), DEFAULT_MAX_LOCKING_DURATION_SEC);

    // error scene 
    // 1 : E100_ACC_NOT_REGISTERED
    assert_err!(
        e.unlock_and_unstake_seed(&users.farmer2, &seed_id, to_yocto("100"), to_yocto("100")),
        E100_ACC_NOT_REGISTERED
    );

    // 2 : E301_SEED_NOT_EXIST
    assert_err!(
        e.unlock_and_unstake_seed(&users.farmer1, &format!("{}#{}", "seed_id", 1), to_yocto("100"), to_yocto("100")),
        E301_SEED_NOT_EXIST
    );

    // 3 : E305_STILL_IN_LOCK
    assert_err!(
        e.unlock_and_unstake_seed(&users.farmer1, &seed_id, to_yocto("100"), to_yocto("100")),
        E305_STILL_IN_LOCK
    );

    e.skip_time(DEFAULT_MAX_LOCKING_DURATION_SEC + 1);

    // 4 : E101_INSUFFICIENT_BALANCE
    assert_err!(
        e.unlock_and_unstake_seed(&users.farmer1, &seed_id, to_yocto("150"), to_yocto("100")),
        E101_INSUFFICIENT_BALANCE
    );

    // 5 : E101_INSUFFICIENT_BALANCE
    assert_err!(
        e.unlock_and_unstake_seed(&users.farmer1, &seed_id, to_yocto("100"), to_yocto("150")),
        E101_INSUFFICIENT_BALANCE
    );

    // // 6 : ERR_RECEIVER_NOT_REGISTERED
    // e.mft_unregister(&token_id, &users.farmer1);
    // assert_err!(
    //     e.unlock_and_unstake_seed(&users.farmer1, &seed_id, to_yocto("100"), to_yocto("25")),
    //     "ERR_RECEIVER_NOT_REGISTERED"
    // );
    // e.mft_storage_deposit(&token_id, &users.farmer1);
    // assert_eq!(e.list_lostfound().get(&seed_id).unwrap().0, to_yocto("25"));

    // success
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id), 0);
    e.unlock_and_unstake_seed(&users.farmer1, &seed_id, to_yocto("100"), to_yocto("100")).assert_success();
    assert!(e.get_farmer_seed(&users.farmer1, &seed_id).is_null());
    assert_seed_farmer_count(e.get_seed(&seed_id), 0);
    // assert_eq!(e.mft_balance_of(&users.farmer1, &token_id), to_yocto("75"));
}

#[test]
fn test_withdraw_seed(){
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
    e.mft_stake_free_seed(&users.farmer1, &token_id, to_yocto("100")).assert_success();
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id), to_yocto("100"), 0, 0, 0, 0);

    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id), 0);
    e.unlock_and_unstake_seed(&users.farmer1, &seed_id, 0, to_yocto("100")).assert_success();
    assert!(e.get_farmer_seed(&users.farmer1, &seed_id).is_null());

    // 1 : E305_STILL_IN_LOCK
    assert_err!(
        e.withdraw_seed(&users.farmer1, &seed_id, Some(to_yocto("30"))),
        E305_STILL_IN_LOCK
    );

    // 2 : E305_STILL_IN_LOCK
    assert_err!(
        e.withdraw_seed(&users.farmer1, &seed_id, None),
        E305_STILL_IN_LOCK
    );

    // pass 1 day
    e.skip_time(3600 * 24 + 1);
    // 3 : E305_STILL_IN_LOCK
    assert_err!(
        e.withdraw_seed(&users.farmer1, &seed_id, Some(to_yocto("30"))),
        E305_STILL_IN_LOCK
    );

    // pass another 6 days
    e.skip_time(3600 * 24 * 6 + 1);
    // 4 : ERR_RECEIVER_NOT_REGISTERED
    e.mft_unregister(&token_id, &users.farmer1);
    assert_err!(
        e.withdraw_seed(&users.farmer1, &seed_id, Some(to_yocto("30"))),
        "ERR_RECEIVER_NOT_REGISTERED"
    );
    e.mft_storage_deposit(&token_id, &users.farmer1);
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id), 0);

    // 5 : E305_STILL_IN_LOCK
    assert_err!(
        e.withdraw_seed(&users.farmer1, &seed_id, Some(to_yocto("30"))),
        E305_STILL_IN_LOCK
    );

    // 6 : pass another 7 days
    e.skip_time(3600 * 24 * 7 + 1);
    e.withdraw_seed(&users.farmer1, &seed_id, Some(to_yocto("30"))).assert_success();
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id), to_yocto("30"));

    e.withdraw_seed(&users.farmer1, &seed_id, None).assert_success();
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id), to_yocto("100"));
}

#[test]
fn test_force_unlock(){
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
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id), 0, to_yocto("50"), to_yocto("100"), e.current_time() + to_nano(DEFAULT_MAX_LOCKING_DURATION_SEC), DEFAULT_MAX_LOCKING_DURATION_SEC);

    // error scene 
    // 1 : E100_ACC_NOT_REGISTERED
    assert_err!(
        e.force_unlock(&users.farmer2, &seed_id, to_yocto("50")),
        E100_ACC_NOT_REGISTERED
    );

    // 2 : E301_SEED_NOT_EXIST
    assert_err!(
        e.force_unlock(&users.farmer1, &format!("{}#{}", "seed_id", 1), to_yocto("50")),
        E301_SEED_NOT_EXIST
    );

    // 3 : E101_INSUFFICIENT_BALANCE
    assert_err!(
        e.force_unlock(&users.farmer1, &seed_id, to_yocto("150")),
        E101_INSUFFICIENT_BALANCE
    );

    e.skip_time(DEFAULT_MAX_LOCKING_DURATION_SEC + 1);

    // 4 : E309_NO_NEED_FORCE
    assert_err!(
        e.force_unlock(&users.farmer1, &seed_id, to_yocto("50")),
        E309_NO_NEED_FORCE
    );

    println!("> farmer1 mft_stake_lock_seed at : {}, when before lock expired", e.current_time());
    let farmer1_unlock_time = e.current_time() + to_nano(DEFAULT_MAX_LOCKING_DURATION_SEC);
    e.mft_stake_lock_seed(&token_id, &users.farmer1, to_yocto("50"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id), 0, to_yocto("100"), to_yocto("200"), farmer1_unlock_time, DEFAULT_MAX_LOCKING_DURATION_SEC);

    e.force_unlock(&users.farmer1, &seed_id, to_yocto("100")).assert_success();
    let full_slashed = u128_ratio(to_yocto("100"), DEFAULT_SEED_SLASH_RATE as u128, BP_DENOM);
    let seed_slashed = u128_ratio(full_slashed, (farmer1_unlock_time - e.current_time()) as u128, to_nano(DEFAULT_MAX_LOCKING_DURATION_SEC) as u128);
    let free_amount = to_yocto("100") - seed_slashed;
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id), free_amount, 0, 0, 0, 0);
    assert_eq!(e.list_slashed().get(&seed_id).unwrap().0, seed_slashed);
}