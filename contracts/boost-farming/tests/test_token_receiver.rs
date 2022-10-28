mod setup;
use crate::setup::*;

#[test]
fn test_reward(){
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
    
    e.ft_mint(&tokens.nref, &users.operator, to_yocto("100"));

    // error scene 
    // 1 : E406_INVALID_FARM_ID
    assert_err!(
        e.deposit_reward(&tokens.nref, &users.operator, to_yocto("100"), &"farm_id".to_string()),
        E406_INVALID_FARM_ID
    );

    // 2 : E301_SEED_NOT_EXIST
    assert_err!(
        e.deposit_reward(&tokens.nref, &users.operator, to_yocto("100"), &format!("{}#{}", "seed_id", 1)),
        E301_SEED_NOT_EXIST
    );

    // 3 : E401_FARM_NOT_EXIST
    assert_err!(
        e.deposit_reward(&tokens.nref, &users.operator, to_yocto("100"), &format!("{}#{}", seed_id, 1)),
        E401_FARM_NOT_EXIST
    );

    // 4 : E404_UNMATCHED_REWARD_TOKEN
    e.ft_mint(&tokens.wnear, &users.operator, to_yocto("10000"));
    assert_err!(
        e.deposit_reward(&tokens.wnear, &users.operator, to_yocto("100"), &farm_id),
        E404_UNMATCHED_REWARD_TOKEN
    );
    
    // success
    println!("> deposit_reward at : {}", e.current_time());
    assert_eq!(e.ft_balance_of(&tokens.nref, &users.operator), to_yocto("100"));
    e.deposit_reward(&tokens.nref, &users.operator, to_yocto("100"), &farm_id).assert_success();
    assert_farm_detail(e.get_farm(&farm_id), to_yocto("100"), start_at, 0, 0, 0, 0, Some(FarmStatus::Created));
}

#[test]
fn test_free(){
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

    println!("> farming register mft at : {}", e.current_time());
    e.mft_storage_deposit(&token_id, &e.farming_contract.user_account);

    // error scene 
    // 1 : E100_ACC_NOT_REGISTERED
    assert_err!(
        e.mft_stake_free_seed(&users.farmer1, &token_id, to_yocto("100")),
        E100_ACC_NOT_REGISTERED
    );

    e.storage_deposit_self_to_farming(&users.farmer1).assert_success();

    // 2 : E307_BELOW_MIN_DEPOSIT
    assert_err!(
        e.mft_stake_free_seed(&users.farmer1, &token_id, 1_000_000_000_000_000_000 - 1),
        E307_BELOW_MIN_DEPOSIT
    );

    // success
    println!("> farmer1 stake_free_seed at : {}", e.current_time());
    e.mft_stake_free_seed(&users.farmer1, &token_id, to_yocto("100")).assert_success();
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id), to_yocto("100"), 0, 0, 0, 0);

}

#[test]
fn test_lock(){
    let e = init_env();
    let users = Users::init(&e);

    let inner_id = "0".to_string();
    let token_id = format!(":{}", inner_id);
    let seed_id = e.mft_seed_id(&inner_id);

    println!("> create_seed at : {}", e.current_time());
    e.create_seed(&e.owner, &seed_id, TOKEN_DECIMALS as u32, None, None).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    
    let inner_id_cant_lock = "1".to_string();
    let token_id_cant_lock = format!(":{}", inner_id_cant_lock);
    let seed_id_cant_lock = e.mft_seed_id(&"1".to_string());
    e.create_seed(&e.owner, &seed_id_cant_lock, TOKEN_DECIMALS as u32, None, Some(0)).assert_success();
    assert_seed(e.get_seed(&seed_id_cant_lock), &seed_id_cant_lock, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, 0);

    println!("> farmer1 mint mft at : {}", e.current_time());
    e.mft_mint(&inner_id, &users.farmer1, to_yocto("100"));
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id), to_yocto("100"));
    e.mft_mint(&inner_id_cant_lock, &users.farmer1, to_yocto("100"));
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id_cant_lock), to_yocto("100"));

    println!("> farming register mft at : {}", e.current_time());
    e.mft_storage_deposit(&token_id, &e.farming_contract.user_account);
    e.mft_storage_deposit(&token_id_cant_lock, &e.farming_contract.user_account);

    // error scene 
    // 1 : E100_ACC_NOT_REGISTERED
    assert_err!(
        e.mft_stake_lock_seed(&token_id, &users.farmer1, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC),
        E100_ACC_NOT_REGISTERED
    );

    e.storage_deposit_self_to_farming(&users.farmer1).assert_success();
    
    // 2 : E307_BELOW_MIN_DEPOSIT
    assert_err!(
        e.mft_stake_lock_seed(&token_id, &users.farmer1, 1_000_000_000_000_000_000 - 1, DEFAULT_MAX_LOCKING_DURATION_SEC),
        E307_BELOW_MIN_DEPOSIT
    );

    // 3 : E201_INVALID_DURATION
    assert_err!(
        e.mft_stake_lock_seed(&token_id, &users.farmer1, 1_000_000_000_000_000_000, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC - 1),
        E201_INVALID_DURATION
    );

    // 4 : E201_INVALID_DURATION
    assert_err!(
        e.mft_stake_lock_seed(&token_id, &users.farmer1, 1_000_000_000_000_000_000, DEFAULT_MAX_LOCKING_DURATION_SEC + 1),
        E201_INVALID_DURATION
    );

    // 5 : E300_FORBID_LOCKING
    assert_err!(
        e.mft_stake_lock_seed(&token_id_cant_lock, &users.farmer1, 1_000_000_000_000_000_000, DEFAULT_MAX_LOCKING_DURATION_SEC),
        E300_FORBID_LOCKING
    );

    // success
    println!("> farmer1 stake_lock_seed at : {}", e.current_time());
    e.mft_stake_lock_seed(&token_id, &users.farmer1, to_yocto("50"), DEFAULT_MAX_LOCKING_DURATION_SEC / 2).assert_success();
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id), 0, to_yocto("50"), to_yocto("75"), e.current_time() + to_nano(DEFAULT_MAX_LOCKING_DURATION_SEC / 2), DEFAULT_MAX_LOCKING_DURATION_SEC / 2);

    // 6 : E304_CAUSE_PRE_UNLOCK
    assert_err!(
        e.mft_stake_lock_seed(&token_id, &users.farmer1, 1_000_000_000_000_000_000, DEFAULT_MAX_LOCKING_DURATION_SEC / 2 - 1),
        E304_CAUSE_PRE_UNLOCK
    );

    // append success
    e.mft_stake_lock_seed(&token_id, &users.farmer1, to_yocto("50"), DEFAULT_MAX_LOCKING_DURATION_SEC);
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id), 0, to_yocto("100"), to_yocto("200"), e.current_time() + to_nano(DEFAULT_MAX_LOCKING_DURATION_SEC), DEFAULT_MAX_LOCKING_DURATION_SEC);
}