mod setup;
use crate::setup::*;
use std::collections::HashMap;
use near_sdk::serde_json::{Value, json};

#[test]
fn test_normal_seed_single_farm(){
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
    println!("> no famer stake, first day reward to beneficiary");
    assert_farm_detail(e.get_farm(&farm_id), to_yocto("100"), e.current_time(), to_yocto("10"), 0, to_yocto("10"), 0, Some(FarmStatus::Running));

    
    println!("> farmer1 mft_stake_free_seed at : {}", e.current_time());
    e.mft_stake_free_seed(&users.farmer1, &token_id, to_yocto("100")).assert_success();
    assert_farm_detail(e.get_farm(&farm_id), to_yocto("100"), e.current_time(), to_yocto("10"), 0, to_yocto("10"), 0, Some(FarmStatus::Running));
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id), to_yocto("100"), 0, 0, 0, 0);
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 1, to_yocto("100"), to_yocto("100"), MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);


    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id), to_yocto("100"), e.current_time(), to_yocto("20"), 0, to_yocto("10"), 0, Some(FarmStatus::Running));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref), to_yocto("10"));


    println!("> farmer1 claim_reward_by_seed at : {}", e.current_time());
    assert_eq!(e.get_farmer_reward(&users.farmer1, &tokens.nref), 0);
    e.claim_reward_by_seed(&users.farmer1, &seed_id).assert_success();
    assert_eq!(e.get_farmer_reward(&users.farmer1, &tokens.nref), to_yocto("10"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref), 0);
    assert_farm_detail(e.get_farm(&farm_id), to_yocto("100"), e.current_time(), to_yocto("20"), to_yocto("10"), to_yocto("10"), 0, Some(FarmStatus::Running));


    println!("> farmer2 mft_stake_lock_seed at : {}", e.current_time());
    
    e.mft_stake_lock_seed(&token_id, &users.farmer2, to_yocto("50"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    let farmer2_unlock_time = e.current_time() + to_nano(DEFAULT_MAX_LOCKING_DURATION_SEC);
    assert_farm_detail(e.get_farm(&farm_id), to_yocto("100"), e.current_time(), to_yocto("20"), to_yocto("10"), to_yocto("10"), 0, Some(FarmStatus::Running));
    assert_user_seed_info(e.get_farmer_seed(&users.farmer2, &seed_id), 0, to_yocto("50"), to_yocto("100"), farmer2_unlock_time, DEFAULT_MAX_LOCKING_DURATION_SEC);
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 1, to_yocto("150"), to_yocto("200"), MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);


    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id), to_yocto("100"), e.current_time(), to_yocto("30"), to_yocto("10"), to_yocto("10"), 0, Some(FarmStatus::Running));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref), to_yocto("5"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.nref), to_yocto("5"));


    println!("> farmer1 withdraw all seed at : {}, meanwhile claim all reward by seed", e.current_time());
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id), 0);
    assert_eq!(true, e.unlock_and_withdraw_seed(&users.farmer1, &seed_id, 0, to_yocto("100")).unwrap_json::<bool>());
    assert!(e.get_farmer_seed(&users.farmer1, &seed_id).is_null());
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id), to_yocto("100"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref), 0);
    assert_eq!(e.get_farmer_reward(&users.farmer1, &tokens.nref), to_yocto("15"));
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 1, to_yocto("50"), to_yocto("100"), MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);


    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id), to_yocto("100"), e.current_time(), to_yocto("40"), to_yocto("15"), to_yocto("10"), 0, Some(FarmStatus::Running));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.nref), to_yocto("15"));


    println!("> farmer1 mft_stake_free_seed again with lock_free_seed at : {}", e.current_time());
    e.mft_stake_free_seed(&users.farmer1, &token_id, to_yocto("75")).assert_success();
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id), to_yocto("75"), 0, 0, 0, 0);
    let farmer1_unlock_time = e.current_time() + to_nano(DEFAULT_MAX_LOCKING_DURATION_SEC);
    e.lock_free_seed(&users.farmer1, &seed_id, DEFAULT_MAX_LOCKING_DURATION_SEC, to_yocto("25")).assert_success();
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id), to_yocto("50"), to_yocto("25"), to_yocto("50"), farmer1_unlock_time, DEFAULT_MAX_LOCKING_DURATION_SEC);


    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id), to_yocto("100"), e.current_time(), to_yocto("50"), to_yocto("15"), to_yocto("10"), 0, Some(FarmStatus::Running));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref), to_yocto("5"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.nref), to_yocto("20"));


    println!("> farmer2 withdraw all ref reward at : {}", e.current_time());
    assert_eq!(e.get_farmer_reward(&users.farmer2, &tokens.nref), 0);
    e.claim_reward_by_seed(&users.farmer2, &seed_id).assert_success();
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.nref), 0);
    assert_eq!(e.get_farmer_reward(&users.farmer2, &tokens.nref), to_yocto("20"));
    assert_err!(
        e.withdraw_reward(&users.farmer2, &tokens.nref, None),
        "The account farmer2.near is not registered"
    );
    e.ft_storage_deposit(&users.farmer2, &tokens.nref);
    assert_eq!(true, e.withdraw_reward(&users.farmer2, &tokens.nref, None).unwrap_json::<bool>());
    assert_eq!(e.get_farmer_reward(&users.farmer2, &tokens.nref), 0);
    assert_eq!(e.ft_balance_of(&tokens.nref, &users.farmer2), to_yocto("20"));


    println!("> farmer2 force_unlock all seed with slashed at : {}", e.current_time());
    assert_err!(
        e.unlock_and_withdraw_seed(&users.farmer2, &seed_id, to_yocto("50"), 0),
        E305_STILL_IN_LOCK
    );
    e.force_unlock(&users.farmer2, &seed_id, to_yocto("50")).assert_success();
    let full_slashed = u128_ratio(to_yocto("50"), DEFAULT_SEED_SLASH_RATE as u128, BP_DENOM);
    let seed_slashed = u128_ratio(full_slashed, (farmer2_unlock_time - e.current_time()) as u128, to_nano(DEFAULT_MAX_LOCKING_DURATION_SEC) as u128);
    let free_amount = to_yocto("50") - seed_slashed;
    let free_amount_part = free_amount - to_yocto("10");
    assert_user_seed_info(e.get_farmer_seed(&users.farmer2, &seed_id), free_amount, 0, 0, 0, 0);
    assert_eq!(true, e.unlock_and_withdraw_seed(&users.farmer2, &seed_id, 0, to_yocto("10")).unwrap_json::<bool>());
    assert_user_seed_info(e.get_farmer_seed(&users.farmer2, &seed_id), free_amount_part, 0, 0, 0, 0);
    assert_eq!(e.mft_balance_of(&users.farmer2, &token_id), to_yocto("60"));
    assert_eq!(true, e.unlock_and_withdraw_seed(&users.farmer2, &seed_id, 0, free_amount_part).unwrap_json::<bool>());
    assert_eq!(e.mft_balance_of(&users.farmer2, &token_id), to_yocto("50") + free_amount);
    e.get_farmer_seed(&users.farmer2, &seed_id).is_null();

    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id), to_yocto("100"), e.current_time(), to_yocto("60"), to_yocto("35"), to_yocto("10"), 0, Some(FarmStatus::Running));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref), to_yocto("15"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.nref), 0);


    println!("> farmer1 force_unlock all seed at : {}, no slashed", e.current_time());
    e.modify_seed_slash_rate(&e.owner, &seed_id, 0, 1).assert_success();
    e.force_unlock(&users.farmer1, &seed_id, to_yocto("25")).assert_success();
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id), to_yocto("75"), 0, 0, 0, 0);



    e.skip_time(to_sec(NANOS_PER_DAY * 5));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id), to_yocto("100"), e.current_time(), to_yocto("100"), to_yocto("50"), to_yocto("10"), 0, Some(FarmStatus::Ended));


    println!("> restarted at : {}", e.current_time());
    e.deposit_reward(&tokens.nref, &users.operator, to_yocto("100"), &farm_id).assert_success();
    assert_farm_detail(e.get_farm(&farm_id), to_yocto("200"), e.current_time(), to_yocto("100"), to_yocto("50"), to_yocto("10"), 0, Some(FarmStatus::Running));
    assert_err!(
        e.remove_farm_from_seed(&e.owner, &farm_id),
        E405_FARM_NOT_ENDED
    );

    e.skip_time(to_sec(NANOS_PER_DAY * 11));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id), to_yocto("200"), e.current_time(), to_yocto("200"), to_yocto("50"), to_yocto("10"), 0, Some(FarmStatus::Ended));
    assert!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref) <= to_yocto("140"));
    assert_eq!(format!("{:?}", e.get_farm(&farm_id)), format!("{:?}", e.list_seed_farms(&seed_id)[0]));
    assert_eq!(format!("{:?}", e.get_seed(&seed_id)), format!("{:?}", e.list_seeds_info().get(0).unwrap()));
    assert_eq!(format!("{:?}", e.get_farmer_seed(&users.farmer1, &seed_id)), format!("{:?}", e.list_farmer_seeds(&users.farmer1).get(seed_id.clone()).unwrap()));
    assert_eq!(format!("{:?}", e.get_farmer_reward(&users.farmer1, &tokens.nref)), format!("{:?}", e.list_farmer_rewards(&users.farmer1).get(&tokens.nref.account_id()).unwrap().0));
    
    e.remove_farm_from_seed(&e.owner, &farm_id).assert_success();
    assert_eq!(e.get_metadata().farmer_count.0, 2);
    assert_eq!(e.get_metadata().farm_count.0, 0);
    assert_eq!(e.get_metadata().outdated_farm_count.0, 1);

    assert_farm_detail(e.get_outdated_farm(&farm_id), to_yocto("200"), e.current_time(), to_yocto("200"), to_yocto("50"), to_yocto("150"), 0, None);
    assert_eq!(format!("{:?}", e.get_outdated_farm(&farm_id)), format!("{:?}", e.list_outdated_farms()[0]));
    

    assert_eq!(e.mft_balance_of(&e.owner, &token_id), 0);
    assert_err!(
        e.withdraw_seed_slashed(&e.owner, &seed_id, 1),
        "ERR_RECEIVER_NOT_REGISTERED"
    );
    assert_eq!(e.list_slashed().get(&seed_id).unwrap().0, seed_slashed);
    e.mft_storage_deposit(&token_id, &e.owner);
    e.withdraw_seed_slashed(&e.owner, &seed_id, 1).assert_success();
    assert_eq!(e.mft_balance_of(&e.owner, &token_id), seed_slashed);
}

#[test]
fn test_normal_mutli_farm(){
    let e = init_env();
    let users = Users::init(&e);
    let tokens = Tokens::init(&e);

    let inner_id = "0".to_string();
    let token_id = format!(":{}", inner_id);
    let seed_id = e.mft_seed_id(&inner_id);
    let farm_id_0 = format!("{}#{}", seed_id, 0);
    let farm_id_1 = format!("{}#{}", seed_id, 1);

    println!("> create_seed at : {}", e.current_time());
    e.create_seed(&e.owner, &seed_id, TOKEN_DECIMALS as u32, None, None).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);


    println!("> farmer1 mint mft at : {}", e.current_time());
    e.mft_mint(&inner_id, &users.farmer1, to_yocto("100"));
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id), to_yocto("100"));

    println!("> farmer1 register farming at : {}", e.current_time());
    e.storage_deposit_self_to_farming(&users.farmer1).assert_success();

    println!("> farmer2 mint mft at : {}", e.current_time());
    e.mft_mint(&inner_id, &users.farmer2, to_yocto("100"));
    assert_eq!(e.mft_balance_of(&users.farmer2, &token_id), to_yocto("100"));

    println!("> farmer2 register farming at : {}", e.current_time());
    e.storage_deposit_self_to_farming(&users.farmer2).assert_success();

    println!("> farming register mft at : {}", e.current_time());
    e.mft_storage_deposit(&token_id, &e.farming_contract.user_account);


    let start_at = e.current_time();
    println!("> create_farm at : {}", e.current_time());
    e.create_farm(&e.owner, &seed_id, &tokens.nref, to_sec(start_at + NANOS_PER_DAY), to_yocto("30")).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 1, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    assert_farm_info(e.get_farm(&farm_id_0), &farm_id_0, &tokens.nref, to_sec(start_at + NANOS_PER_DAY), to_yocto("30"));
    assert_farm_detail(e.get_farm(&farm_id_0), 0, start_at + NANOS_PER_DAY, 0, 0, 0, 0, Some(FarmStatus::Created));

    e.create_farm(&e.owner, &seed_id, &tokens.wnear, to_sec(start_at), to_yocto("15")).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 2, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    assert_farm_info(e.get_farm(&farm_id_1), &farm_id_1, &tokens.wnear, to_sec(start_at), to_yocto("15"));
    assert_farm_detail(e.get_farm(&farm_id_1), 0, start_at, 0, 0, 0, 0, Some(FarmStatus::Created));


    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id_0), 0, e.current_time(), 0, 0, 0, 0, Some(FarmStatus::Created));
    assert_farm_detail(e.get_farm(&farm_id_1), 0, e.current_time() - NANOS_PER_DAY, 0, 0, 0, 0, Some(FarmStatus::Pending));

    
    println!("> deposit_reward at : {}", e.current_time());
    e.ft_mint(&tokens.nref, &users.operator, to_yocto("10000"));
    assert_eq!(e.ft_balance_of(&tokens.nref, &users.operator), to_yocto("10000"));
    e.deposit_reward(&tokens.nref, &users.operator, to_yocto("300"), &farm_id_0).assert_success();
    assert_farm_detail(e.get_farm(&farm_id_0), to_yocto("300"), e.current_time(), 0, 0, 0, 0, Some(FarmStatus::Created));
    
    e.ft_mint(&tokens.wnear, &users.operator, to_yocto("10000"));
    assert_eq!(e.ft_balance_of(&tokens.wnear, &users.operator), to_yocto("10000"));
    e.deposit_reward(&tokens.wnear, &users.operator, to_yocto("150"), &farm_id_1).assert_success();
    assert_farm_detail(e.get_farm(&farm_id_1), to_yocto("150"), e.current_time(), to_yocto("15"), 0, to_yocto("15"), 0, Some(FarmStatus::Running));
    

    println!("> farmer1 mft_stake_free_seed at : {}", e.current_time());
    e.mft_stake_free_seed(&users.farmer1, &token_id, to_yocto("100")).assert_success();
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id), to_yocto("100"), 0, 0, 0, 0);
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 2, to_yocto("100"), to_yocto("100"), MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);


    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id_0), to_yocto("300"), e.current_time(), to_yocto("30"), 0, 0, 0, Some(FarmStatus::Running));
    assert_farm_detail(e.get_farm(&farm_id_1), to_yocto("150"), e.current_time(), to_yocto("30"), 0, to_yocto("15"), 0, Some(FarmStatus::Running));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref), to_yocto("30"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.wnear), to_yocto("15"));


    println!("> farmer2 mft_stake_lock_seed at : {}", e.current_time());
    
    e.mft_stake_lock_seed(&token_id, &users.farmer2, to_yocto("50"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    let farmer2_unlock_time = to_nano(DEFAULT_MAX_LOCKING_DURATION_SEC) + e.current_time();
    assert_user_seed_info(e.get_farmer_seed(&users.farmer2, &seed_id), 0, to_yocto("50"), to_yocto("100"), farmer2_unlock_time, DEFAULT_MAX_LOCKING_DURATION_SEC);
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 2, to_yocto("150"), to_yocto("200"), MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);


    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id_0), to_yocto("300"), e.current_time(), to_yocto("60"), 0, 0, 0, Some(FarmStatus::Running));
    assert_farm_detail(e.get_farm(&farm_id_1), to_yocto("150"), e.current_time(), to_yocto("45"), 0, to_yocto("15"), 0, Some(FarmStatus::Running));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref), to_yocto("45"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.wnear), to_yocto("22.5"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.nref), to_yocto("15"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.wnear), to_yocto("7.5"));


    println!("> farmer1 claim_reward_by_seed at : {}", e.current_time());
    assert_eq!(e.get_farmer_reward(&users.farmer1, &tokens.nref), 0);
    assert_eq!(e.get_farmer_reward(&users.farmer1, &tokens.wnear), 0);
    e.claim_reward_by_seed(&users.farmer1, &seed_id).assert_success();
    assert_eq!(e.get_farmer_reward(&users.farmer1, &tokens.nref), to_yocto("45"));
    assert_eq!(e.get_farmer_reward(&users.farmer1, &tokens.wnear), to_yocto("22.5"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref), 0);
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.wnear), 0);
    assert_farm_detail(e.get_farm(&farm_id_0), to_yocto("300"), e.current_time(), to_yocto("60"), to_yocto("45"), 0, 0, Some(FarmStatus::Running));
    assert_farm_detail(e.get_farm(&farm_id_1), to_yocto("150"), e.current_time(), to_yocto("45"), to_yocto("22.5"), to_yocto("15"), 0, Some(FarmStatus::Running));


    println!("> farmer1 withdraw all seed at : {}", e.current_time());
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id), 0);
    assert_eq!(true, e.unlock_and_withdraw_seed(&users.farmer1, &seed_id, 0, to_yocto("100")).unwrap_json::<bool>());
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id), to_yocto("100"));
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 2, to_yocto("50"), to_yocto("100"), MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);


    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id_0), to_yocto("300"), e.current_time(), to_yocto("90"), to_yocto("45"), 0, 0, Some(FarmStatus::Running));
    assert_farm_detail(e.get_farm(&farm_id_1), to_yocto("150"), e.current_time(), to_yocto("60"), to_yocto("22.5"), to_yocto("15"), 0, Some(FarmStatus::Running));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref), 0);
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.wnear), 0);
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.nref), to_yocto("45"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.wnear), to_yocto("22.5"));


    println!("> farmer1 mft_stake_free_seed again with lock_free_seed at : {}", e.current_time());
    e.mft_stake_free_seed(&users.farmer1, &token_id, to_yocto("50")).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 2, to_yocto("100"), to_yocto("150"), MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id), to_yocto("50"), 0, 0, 0, 0);
    let farmer1_unlock_time = e.current_time() + to_nano(DEFAULT_MAX_LOCKING_DURATION_SEC);
    e.lock_free_seed(&users.farmer1, &seed_id, DEFAULT_MAX_LOCKING_DURATION_SEC, to_yocto("50")).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 2, to_yocto("100"), to_yocto("200"), MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id), 0, to_yocto("50"), to_yocto("100"), farmer1_unlock_time, DEFAULT_MAX_LOCKING_DURATION_SEC);


    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id_0), to_yocto("300"), e.current_time(), to_yocto("120"), to_yocto("45"), 0, 0, Some(FarmStatus::Running));
    assert_farm_detail(e.get_farm(&farm_id_1), to_yocto("150"), e.current_time(), to_yocto("75"), to_yocto("22.5"), to_yocto("15"), 0, Some(FarmStatus::Running));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref), to_yocto("15"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.wnear), to_yocto("7.5"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.nref), to_yocto("60"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.wnear), to_yocto("30"));


    println!("> farmer2 force_unlock all seed with slashed at : {}, meanwhile claim all reward by seed", e.current_time());
    e.force_unlock(&users.farmer2, &seed_id, to_yocto("50")).assert_success();
    let full_slashed = u128_ratio(to_yocto("50"), DEFAULT_SEED_SLASH_RATE as u128, BP_DENOM);
    let seed_slashed = u128_ratio(full_slashed, (farmer2_unlock_time - e.current_time()) as u128, to_nano(DEFAULT_MAX_LOCKING_DURATION_SEC) as u128);
    let free_amount = to_yocto("50") - seed_slashed;
    assert_user_seed_info(e.get_farmer_seed(&users.farmer2, &seed_id), free_amount, 0, 0, 0, 0);
    assert_eq!(true, e.unlock_and_withdraw_seed(&users.farmer2, &seed_id, 0, free_amount).unwrap_json::<bool>());
    assert_eq!(e.mft_balance_of(&users.farmer2, &token_id), to_yocto("50") + free_amount);
    e.get_farmer_seed(&users.farmer2, &seed_id).is_null();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 2, to_yocto("50"), to_yocto("100"), MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.nref), 0);
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.wnear), 0);
    assert_eq!(e.get_farmer_reward(&users.farmer2, &tokens.nref), to_yocto("60"));
    assert_eq!(e.get_farmer_reward(&users.farmer2, &tokens.wnear), to_yocto("30"));
    e.ft_storage_deposit(&users.farmer2, &tokens.nref);
    e.ft_storage_deposit(&users.farmer2, &tokens.wnear);
    assert_eq!(true, e.withdraw_reward(&users.farmer2, &tokens.nref, None).unwrap_json::<bool>());
    assert_eq!(true, e.withdraw_reward(&users.farmer2, &tokens.wnear, None).unwrap_json::<bool>());
    assert_eq!(e.get_farmer_reward(&users.farmer2, &tokens.nref), 0);
    assert_eq!(e.get_farmer_reward(&users.farmer2, &tokens.wnear), 0);
    assert_eq!(e.ft_balance_of(&tokens.nref, &users.farmer2), to_yocto("60"));
    assert_eq!(e.ft_balance_of(&tokens.wnear, &users.farmer2), to_yocto("30"));


    e.skip_time(DEFAULT_MAX_LOCKING_DURATION_SEC);
    println!(">> time pass {}, now at : {}", to_nano(DEFAULT_MAX_LOCKING_DURATION_SEC), e.current_time());
    assert_farm_detail(e.get_farm(&farm_id_0), to_yocto("300"), e.current_time(), to_yocto("300"), to_yocto("105"), 0, 0, Some(FarmStatus::Ended));
    assert_farm_detail(e.get_farm(&farm_id_1), to_yocto("150"), e.current_time(), to_yocto("150"), to_yocto("52.5"), to_yocto("15"), 0, Some(FarmStatus::Ended));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref), to_yocto("195"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.wnear), to_yocto("82.5"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.nref), 0);
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.wnear), 0);


    println!("> farmer1 unclock and withdraw all seed at : {}, meanwhile claim all reward by seed", e.current_time());
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id), to_yocto("50"));
    assert_eq!(true, e.unlock_and_withdraw_seed(&users.farmer1, &seed_id, to_yocto("50"), to_yocto("50")).unwrap_json::<bool>());
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id), to_yocto("100"));
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 2, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    assert_farm_detail(e.get_farm(&farm_id_0), to_yocto("300"), e.current_time(), to_yocto("300"), to_yocto("300"), 0, 0, Some(FarmStatus::Ended));
    assert_farm_detail(e.get_farm(&farm_id_1), to_yocto("150"), e.current_time(), to_yocto("150"), to_yocto("135"), to_yocto("15"), 0, Some(FarmStatus::Ended));


    e.skip_time(DEFAULT_MAX_LOCKING_DURATION_SEC);
    println!(">> time pass {}, now at : {}", to_nano(DEFAULT_MAX_LOCKING_DURATION_SEC), e.current_time());


    println!("> restarted farm_id_0 at : {}", e.current_time());
    e.deposit_reward(&tokens.nref, &users.operator, to_yocto("75"), &farm_id_0).assert_success();
    assert_farm_detail(e.get_farm(&farm_id_0), to_yocto("375"), e.current_time(), to_yocto("300"), to_yocto("300"), 0, 0, Some(FarmStatus::Running));
    assert_farm_detail(e.get_farm(&farm_id_1), to_yocto("150"), e.current_time(), to_yocto("150"), to_yocto("135"), to_yocto("15"), 0, Some(FarmStatus::Ended));


    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id_0), to_yocto("375"), e.current_time(), to_yocto("330"), to_yocto("300"), to_yocto("30"), 0, Some(FarmStatus::Running));
    assert_farm_detail(e.get_farm(&farm_id_1), to_yocto("150"), e.current_time(), to_yocto("150"), to_yocto("135"), to_yocto("15"), 0, Some(FarmStatus::Ended));


    println!("> farmer1 and farmer2 mft_stake_free_seed at : {}", e.current_time());
    e.mft_stake_free_seed(&users.farmer1, &token_id, to_yocto("100")).assert_success();
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id), to_yocto("100"), 0, 0, 0, 0);
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 2, to_yocto("100"), to_yocto("100"), MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    e.mft_stake_free_seed(&users.farmer2, &token_id, to_yocto("50")).assert_success();
    assert_user_seed_info(e.get_farmer_seed(&users.farmer2, &seed_id), to_yocto("50"), 0, 0, 0, 0);
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 2, to_yocto("150"), to_yocto("150"), MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);


    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id_0), to_yocto("375"), e.current_time(), to_yocto("360"), to_yocto("300"), to_yocto("30"), 0, Some(FarmStatus::Running));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref), to_yocto("20"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.nref), to_yocto("10"));

    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}, Insufficient reward, send all", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id_0), to_yocto("375"), e.current_time(), to_yocto("375"), to_yocto("300"), to_yocto("30"), 0, Some(FarmStatus::Ended));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref), to_yocto("30"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.nref), to_yocto("15"));
}


#[test]
fn test_mutli_seed_with_booster_and_normal(){
    let e = init_env();
    let users = Users::init(&e);
    let tokens = Tokens::init(&e);

    let inner_id_booster = "0".to_string();
    let token_id_booster = format!(":{}", inner_id_booster);
    let seed_id_booster = e.mft_seed_id(&inner_id_booster);
    let farm_id_0_booster = format!("{}#{}", seed_id_booster, 0);
    let farm_id_1_booster = format!("{}#{}", seed_id_booster, 1);

    println!("> create seed affected by booster seed at : {}", e.current_time());
    e.create_seed(&e.owner, &seed_id_booster, TOKEN_DECIMALS as u32, None, None).assert_success();
    assert_seed(e.get_seed(&seed_id_booster), &seed_id_booster, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);

    let inner_id_normal = "1".to_string();
    let token_id_normal = format!(":{}", inner_id_normal);
    let seed_id_normal = e.mft_seed_id(&inner_id_normal);
    let farm_id_0_normal = format!("{}#{}", seed_id_normal, 0);
    let farm_id_1_normal = format!("{}#{}", seed_id_normal, 1);

    println!("> create seed not affected by booster seed at : {}", e.current_time());
    e.create_seed(&e.owner, &seed_id_normal, TOKEN_DECIMALS as u32, None, None).assert_success();
    assert_seed(e.get_seed(&seed_id_normal), &seed_id_normal, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);

    println!("booster> create booster seed at : {}, min_locking_duration_sec is 0, can not lock", e.current_time());
    let booster_id = tokens.love_ref.account_id.to_string();
    let mut affected_seeds = HashMap::new();
    affected_seeds.insert(seed_id_booster.clone(), 10);
    let booster_info = BoosterInfo { booster_decimal: 24, affected_seeds: affected_seeds.clone()};
    e.create_seed(&e.owner, &booster_id, TOKEN_DECIMALS as u32, None, Some(0)).assert_success();
    assert_seed(e.get_seed(&booster_id), &booster_id, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, 0);
    e.modify_booster(&e.owner, &booster_id, &booster_info).assert_success();

    println!("> farmer1 mint mft at : {}", e.current_time());
    e.mft_mint(&inner_id_normal, &users.farmer1, to_yocto("100"));
    e.mft_mint(&inner_id_booster, &users.farmer1, to_yocto("100"));
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id_normal), to_yocto("100"));
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id_booster), to_yocto("100"));

    println!("> farmer1 register farming at : {}", e.current_time());
    e.storage_deposit_self_to_farming(&users.farmer1).assert_success();

    println!("> farmer2 mint mft at : {}", e.current_time());
    e.mft_mint(&inner_id_normal, &users.farmer2, to_yocto("100"));
    e.mft_mint(&inner_id_booster, &users.farmer2, to_yocto("100"));
    assert_eq!(e.mft_balance_of(&users.farmer2, &token_id_normal), to_yocto("100"));
    assert_eq!(e.mft_balance_of(&users.farmer2, &token_id_booster), to_yocto("100"));


    println!("> farmer2 register farming at : {}", e.current_time());
    e.storage_deposit_self_to_farming(&users.farmer2).assert_success();

    println!("> farming register mft at : {}", e.current_time());
    e.mft_storage_deposit(&token_id_normal, &e.farming_contract.user_account);
    e.mft_storage_deposit(&token_id_booster, &e.farming_contract.user_account);


    let start_at = e.current_time();
    println!("normal> create farm at : {}", e.current_time());
    e.create_farm(&e.owner, &seed_id_normal, &tokens.nref, to_sec(start_at + NANOS_PER_DAY), to_yocto("30")).assert_success();
    assert_seed(e.get_seed(&seed_id_normal), &seed_id_normal, TOKEN_DECIMALS as u32, 1, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    assert_farm_info(e.get_farm(&farm_id_0_normal), &farm_id_0_normal, &tokens.nref, to_sec(start_at + NANOS_PER_DAY), to_yocto("30"));
    assert_farm_detail(e.get_farm(&farm_id_0_normal), 0, start_at + NANOS_PER_DAY, 0, 0, 0, 0, Some(FarmStatus::Created));

    e.create_farm(&e.owner, &seed_id_normal, &tokens.wnear, to_sec(start_at), to_yocto("15")).assert_success();
    assert_seed(e.get_seed(&seed_id_normal), &seed_id_normal, TOKEN_DECIMALS as u32, 2, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    assert_farm_info(e.get_farm(&farm_id_1_normal), &farm_id_1_normal, &tokens.wnear, to_sec(start_at), to_yocto("15"));
    assert_farm_detail(e.get_farm(&farm_id_1_normal), 0, start_at, 0, 0, 0, 0, Some(FarmStatus::Created));


    println!("booster> create farm at : {}", e.current_time());
    e.create_farm(&e.owner, &seed_id_booster, &tokens.neth, to_sec(start_at + NANOS_PER_DAY), to_yocto("30")).assert_success();
    assert_seed(e.get_seed(&seed_id_booster), &seed_id_booster, TOKEN_DECIMALS as u32, 1, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    assert_farm_info(e.get_farm(&farm_id_0_booster), &farm_id_0_booster, &tokens.neth, to_sec(start_at + NANOS_PER_DAY), to_yocto("30"));
    assert_farm_detail(e.get_farm(&farm_id_0_booster), 0, start_at + NANOS_PER_DAY, 0, 0, 0, 0, Some(FarmStatus::Created));

    e.create_farm(&e.owner, &seed_id_booster, &tokens.ndai, to_sec(start_at), to_yocto("15")).assert_success();
    assert_seed(e.get_seed(&seed_id_booster), &seed_id_booster, TOKEN_DECIMALS as u32, 2, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    assert_farm_info(e.get_farm(&farm_id_1_booster), &farm_id_1_booster, &tokens.ndai, to_sec(start_at), to_yocto("15"));
    assert_farm_detail(e.get_farm(&farm_id_1_booster), 0, start_at, 0, 0, 0, 0, Some(FarmStatus::Created));

    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id_0_normal), 0, e.current_time(), 0, 0, 0, 0, Some(FarmStatus::Created));
    assert_farm_detail(e.get_farm(&farm_id_1_normal), 0, e.current_time() - NANOS_PER_DAY, 0, 0, 0, 0, Some(FarmStatus::Pending));
    assert_farm_detail(e.get_farm(&farm_id_0_booster), 0, e.current_time(), 0, 0, 0, 0, Some(FarmStatus::Created));
    assert_farm_detail(e.get_farm(&farm_id_1_booster), 0, e.current_time() - NANOS_PER_DAY, 0, 0, 0, 0, Some(FarmStatus::Pending));

    
    println!("normal> deposit reward at : {}", e.current_time());
    e.ft_mint(&tokens.nref, &users.operator, to_yocto("10000"));
    e.ft_mint(&tokens.neth, &users.operator, to_yocto("10000"));
    assert_eq!(e.ft_balance_of(&tokens.nref, &users.operator), to_yocto("10000"));
    assert_eq!(e.ft_balance_of(&tokens.neth, &users.operator), to_yocto("10000"));
    e.deposit_reward(&tokens.nref, &users.operator, to_yocto("300"), &farm_id_0_normal).assert_success();
    assert_farm_detail(e.get_farm(&farm_id_0_normal), to_yocto("300"), e.current_time(), 0, 0, 0, 0, Some(FarmStatus::Created));
    e.deposit_reward(&tokens.neth, &users.operator, to_yocto("300"), &farm_id_0_booster).assert_success();
    assert_farm_detail(e.get_farm(&farm_id_0_booster), to_yocto("300"), e.current_time(), 0, 0, 0, 0, Some(FarmStatus::Created));
    
    println!("booster> deposit reward at : {}", e.current_time());
    e.ft_mint(&tokens.wnear, &users.operator, to_yocto("10000"));
    e.ft_mint(&tokens.ndai, &users.operator, to_yocto("10000"));
    assert_eq!(e.ft_balance_of(&tokens.wnear, &users.operator), to_yocto("10000"));
    assert_eq!(e.ft_balance_of(&tokens.ndai, &users.operator), to_yocto("10000"));
    e.deposit_reward(&tokens.wnear, &users.operator, to_yocto("150"), &farm_id_1_normal).assert_success();
    assert_farm_detail(e.get_farm(&farm_id_1_normal), to_yocto("150"), e.current_time(), to_yocto("15"), 0, to_yocto("15"), 0, Some(FarmStatus::Running));
    e.deposit_reward(&tokens.ndai, &users.operator, to_yocto("150"), &farm_id_1_booster).assert_success();
    assert_farm_detail(e.get_farm(&farm_id_1_booster), to_yocto("150"), e.current_time(), to_yocto("15"), 0, to_yocto("15"), 0, Some(FarmStatus::Running));

    println!("normal> farmer1 mft_stake_free_seed  at : {}", e.current_time());
    e.mft_stake_free_seed(&users.farmer1, &token_id_normal, to_yocto("100")).assert_success();
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id_normal), to_yocto("100"), 0, 0, 0, 0);
    assert_seed(e.get_seed(&seed_id_normal), &seed_id_normal, TOKEN_DECIMALS as u32, 2, to_yocto("100"), to_yocto("100"), MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);

    println!("booster> farmer1 mft_stake_free_seed at : {}", e.current_time());
    e.mft_stake_free_seed(&users.farmer1, &token_id_booster, to_yocto("100")).assert_success();
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id_booster), to_yocto("100"), 0, 0, 0, 0);
    assert_seed(e.get_seed(&seed_id_booster), &seed_id_booster, TOKEN_DECIMALS as u32, 2, to_yocto("100"), to_yocto("100"), MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);

    println!("booster> farmer1 stake booster token at : {}", e.current_time());
    e.ft_mint(&tokens.love_ref, &users.farmer1, to_yocto("10"));
    e.ft_stake_free_seed(&users.farmer1, &tokens.love_ref, to_yocto("10")).assert_success();
    assert_eq!(e.get_farmer_seed(&users.farmer1, &seed_id_booster).get("boost_ratios").unwrap()[booster_id.clone()], json!(1.0));
    assert_seed(e.get_seed(&seed_id_booster), &seed_id_booster, TOKEN_DECIMALS as u32, 2, to_yocto("100"), 200000000000000004764729344, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    
    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id_0_normal), to_yocto("300"), e.current_time(), to_yocto("30"), 0, 0, 0, Some(FarmStatus::Running));
    assert_farm_detail(e.get_farm(&farm_id_1_normal), to_yocto("150"), e.current_time(), to_yocto("30"), 0, to_yocto("15"), 0, Some(FarmStatus::Running));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id_normal, &tokens.nref), to_yocto("30"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id_normal, &tokens.wnear), to_yocto("15"));
    assert_farm_detail(e.get_farm(&farm_id_0_booster), to_yocto("300"), e.current_time(), to_yocto("30"), 0, 0, 0, Some(FarmStatus::Running));
    assert_farm_detail(e.get_farm(&farm_id_1_booster), to_yocto("150"), e.current_time(), to_yocto("30"), 0, to_yocto("15"), 0, Some(FarmStatus::Running));
    assert!(e.get_unclaimed_rewards(&users.farmer1, &seed_id_booster, &tokens.neth) <= to_yocto("30"));
    assert!(e.get_unclaimed_rewards(&users.farmer1, &seed_id_booster, &tokens.ndai) <= to_yocto("15"));


    println!("booster> farmer2 stake a part of booster token at : {}", e.current_time());
    e.ft_mint(&tokens.love_ref, &users.farmer2, to_yocto("10"));
    e.ft_stake_free_seed(&users.farmer2, &tokens.love_ref, to_yocto("1")).assert_success();

    println!("normal> farmer2 mft_stake_lock_seed at : {}", e.current_time());
    e.mft_stake_lock_seed(&token_id_normal, &users.farmer2, to_yocto("50"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    let farmer2_unlock_time = to_nano(DEFAULT_MAX_LOCKING_DURATION_SEC) + e.current_time();
    assert_user_seed_info(e.get_farmer_seed(&users.farmer2, &seed_id_normal), 0, to_yocto("50"), to_yocto("100"), farmer2_unlock_time, DEFAULT_MAX_LOCKING_DURATION_SEC);
    assert_seed(e.get_seed(&seed_id_normal), &seed_id_normal, TOKEN_DECIMALS as u32, 2, to_yocto("150"), to_yocto("200"), MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);

    println!("booster> farmer2 mft_stake_lock_seed at : {}", e.current_time());
    e.mft_stake_lock_seed(&token_id_booster, &users.farmer2, to_yocto("50"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    assert_eq!(e.get_farmer_seed(&users.farmer2, &seed_id_booster).get("boost_ratios").unwrap()[booster_id.clone()], json!(0.0));
    assert_user_seed_info(e.get_farmer_seed(&users.farmer2, &seed_id_booster), 0, to_yocto("50"), to_yocto("100"), farmer2_unlock_time, DEFAULT_MAX_LOCKING_DURATION_SEC);
    assert_seed(e.get_seed(&seed_id_booster), &seed_id_booster, TOKEN_DECIMALS as u32, 2, to_yocto("150"), 300000000000000004764729344, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    
    println!("booster> farmer2 stake a part of booster token again  at : {}", e.current_time());
    e.ft_stake_free_seed(&users.farmer2, &tokens.love_ref, to_yocto("9")).assert_success();
    assert_seed(e.get_seed(&seed_id_booster), &seed_id_booster, TOKEN_DECIMALS as u32, 2, to_yocto("150"), 400000000000000009529458688, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    assert_eq!(e.get_farmer_seed(&users.farmer2, &seed_id_booster).get("boost_ratios").unwrap()[booster_id.clone()], json!(1.0));

    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id_0_normal), to_yocto("300"), e.current_time(), to_yocto("60"), 0, 0, 0, Some(FarmStatus::Running));
    assert_farm_detail(e.get_farm(&farm_id_1_normal), to_yocto("150"), e.current_time(), to_yocto("45"), 0, to_yocto("15"), 0, Some(FarmStatus::Running));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id_normal, &tokens.nref), to_yocto("45"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id_normal, &tokens.wnear), to_yocto("22.5"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id_normal, &tokens.nref), to_yocto("15"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id_normal, &tokens.wnear), to_yocto("7.5"));

    assert_farm_detail(e.get_farm(&farm_id_0_booster), to_yocto("300"), e.current_time(), to_yocto("60"), 0, 0, 0, Some(FarmStatus::Running));
    assert_farm_detail(e.get_farm(&farm_id_1_booster), to_yocto("150"), e.current_time(), to_yocto("45"), 0, to_yocto("15"), 0, Some(FarmStatus::Running));
    assert!(e.get_unclaimed_rewards(&users.farmer1, &seed_id_booster, &tokens.neth) <= to_yocto("45"));
    assert!(e.get_unclaimed_rewards(&users.farmer1, &seed_id_booster, &tokens.ndai) <= to_yocto("22.5"));
    assert!(e.get_unclaimed_rewards(&users.farmer2, &seed_id_booster, &tokens.neth) <= to_yocto("15"));
    assert!(e.get_unclaimed_rewards(&users.farmer2, &seed_id_booster, &tokens.ndai) <= to_yocto("7.5"));


    println!("normal> farmer1 claim_reward_by_seed at : {}", e.current_time());
    assert_eq!(e.get_farmer_reward(&users.farmer1, &tokens.nref), 0);
    assert_eq!(e.get_farmer_reward(&users.farmer1, &tokens.wnear), 0);
    e.claim_reward_by_seed(&users.farmer1, &seed_id_normal).assert_success();
    assert_eq!(e.get_farmer_reward(&users.farmer1, &tokens.nref), to_yocto("45"));
    assert_eq!(e.get_farmer_reward(&users.farmer1, &tokens.wnear), to_yocto("22.5"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id_normal, &tokens.nref), 0);
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id_normal, &tokens.wnear), 0);
    assert_farm_detail(e.get_farm(&farm_id_0_normal), to_yocto("300"), e.current_time(), to_yocto("60"), to_yocto("45"), 0, 0, Some(FarmStatus::Running));
    assert_farm_detail(e.get_farm(&farm_id_1_normal), to_yocto("150"), e.current_time(), to_yocto("45"), to_yocto("22.5"), to_yocto("15"), 0, Some(FarmStatus::Running));

    println!("booster> farmer1 claim_reward_by_seed at : {}", e.current_time());
    assert_eq!(e.get_farmer_reward(&users.farmer1, &tokens.neth), 0);
    assert_eq!(e.get_farmer_reward(&users.farmer1, &tokens.ndai), 0);
    let mut farm_id_0_booster_reward_claim = e.get_unclaimed_rewards(&users.farmer1, &seed_id_booster, &tokens.neth);
    let mut farm_id_1_booster_reward_claim = e.get_unclaimed_rewards(&users.farmer1, &seed_id_booster, &tokens.ndai);
    e.claim_reward_by_seed(&users.farmer1, &seed_id_booster).assert_success();
    assert!(e.get_farmer_reward(&users.farmer1, &tokens.neth) <= to_yocto("45"));
    assert!(e.get_farmer_reward(&users.farmer1, &tokens.ndai) <= to_yocto("22.5"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id_booster, &tokens.neth), 0);
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id_booster, &tokens.ndai), 0);
    assert_farm_detail(e.get_farm(&farm_id_0_booster), to_yocto("300"), e.current_time(), to_yocto("60"), farm_id_0_booster_reward_claim, 0, 0, Some(FarmStatus::Running));
    assert_farm_detail(e.get_farm(&farm_id_1_booster), to_yocto("150"), e.current_time(), to_yocto("45"), farm_id_1_booster_reward_claim, to_yocto("15"), 0, Some(FarmStatus::Running));


    println!("nomal> farmer1 withdraw all seed at : {}", e.current_time());
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id_normal), 0);
    assert_eq!(true, e.unlock_and_withdraw_seed(&users.farmer1, &seed_id_normal, 0, to_yocto("100")).unwrap_json::<bool>());
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id_normal), to_yocto("100"));
    assert_seed(e.get_seed(&seed_id_normal), &seed_id_normal, TOKEN_DECIMALS as u32, 2, to_yocto("50"), to_yocto("100"), MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);

    println!("booster> farmer1 withdraw all seed at : {}", e.current_time());
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id_booster), 0);
    assert_eq!(true, e.unlock_and_withdraw_seed(&users.farmer1, &seed_id_booster, 0, to_yocto("100")).unwrap_json::<bool>());
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id_booster), to_yocto("100"));
    assert_seed(e.get_seed(&seed_id_booster), &seed_id_booster, TOKEN_DECIMALS as u32, 2, to_yocto("50"), 200000000000000004764729344, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);

    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id_0_normal), to_yocto("300"), e.current_time(), to_yocto("90"), to_yocto("45"), 0, 0, Some(FarmStatus::Running));
    assert_farm_detail(e.get_farm(&farm_id_1_normal), to_yocto("150"), e.current_time(), to_yocto("60"), to_yocto("22.5"), to_yocto("15"), 0, Some(FarmStatus::Running));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id_normal, &tokens.nref), 0);
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id_normal, &tokens.wnear), 0);
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id_normal, &tokens.nref), to_yocto("45"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id_normal, &tokens.wnear), to_yocto("22.5"));

    assert_farm_detail(e.get_farm(&farm_id_0_booster), to_yocto("300"), e.current_time(), to_yocto("90"), farm_id_0_booster_reward_claim, 0, 0, Some(FarmStatus::Running));
    assert_farm_detail(e.get_farm(&farm_id_1_booster), to_yocto("150"), e.current_time(), to_yocto("60"), farm_id_1_booster_reward_claim, to_yocto("15"), 0, Some(FarmStatus::Running));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id_booster, &tokens.neth), 0);
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id_booster, &tokens.ndai), 0);
    assert!(e.get_unclaimed_rewards(&users.farmer2, &seed_id_booster, &tokens.neth) <= to_yocto("45"));
    assert!(e.get_unclaimed_rewards(&users.farmer2, &seed_id_booster, &tokens.ndai) < to_yocto("22.5"));


    println!("normal> farmer1 mft_stake_free_seed again with lock_free_seed at : {}", e.current_time());
    e.mft_stake_free_seed(&users.farmer1, &token_id_normal, to_yocto("50")).assert_success();
    assert_seed(e.get_seed(&seed_id_normal), &seed_id_normal, TOKEN_DECIMALS as u32, 2, to_yocto("100"), to_yocto("150"), MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id_normal), to_yocto("50"), 0, 0, 0, 0);
    let farmer1_unlock_time = e.current_time() + to_nano(DEFAULT_MAX_LOCKING_DURATION_SEC);
    e.lock_free_seed(&users.farmer1, &seed_id_normal, DEFAULT_MAX_LOCKING_DURATION_SEC, to_yocto("50")).assert_success();
    assert_seed(e.get_seed(&seed_id_normal), &seed_id_normal, TOKEN_DECIMALS as u32, 2, to_yocto("100"), to_yocto("200"), MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id_normal), 0, to_yocto("50"), to_yocto("100"), farmer1_unlock_time, DEFAULT_MAX_LOCKING_DURATION_SEC);

    println!("booster> farmer1 mft_stake_free_seed again with lock_free_seed at : {}", e.current_time());
    e.mft_stake_free_seed(&users.farmer1, &token_id_booster, to_yocto("50")).assert_success();
    assert_seed(e.get_seed(&seed_id_booster), &seed_id_booster, TOKEN_DECIMALS as u32, 2, to_yocto("100"), 300000000000000007147094016, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id_booster), to_yocto("50"), 0, 0, 0, 0);
    let farmer1_unlock_time = e.current_time() + to_nano(DEFAULT_MAX_LOCKING_DURATION_SEC);
    e.lock_free_seed(&users.farmer1, &seed_id_booster, DEFAULT_MAX_LOCKING_DURATION_SEC, to_yocto("50")).assert_success();
    assert_seed(e.get_seed(&seed_id_booster), &seed_id_booster, TOKEN_DECIMALS as u32, 2, to_yocto("100"), 400000000000000009529458688, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id_booster), 0, to_yocto("50"), to_yocto("100"), farmer1_unlock_time, DEFAULT_MAX_LOCKING_DURATION_SEC);


    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id_0_normal), to_yocto("300"), e.current_time(), to_yocto("120"), to_yocto("45"), 0, 0, Some(FarmStatus::Running));
    assert_farm_detail(e.get_farm(&farm_id_1_normal), to_yocto("150"), e.current_time(), to_yocto("75"), to_yocto("22.5"), to_yocto("15"), 0, Some(FarmStatus::Running));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id_normal, &tokens.nref), to_yocto("15"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id_normal, &tokens.wnear), to_yocto("7.5"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id_normal, &tokens.nref), to_yocto("60"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id_normal, &tokens.wnear), to_yocto("30"));

    assert_farm_detail(e.get_farm(&farm_id_0_booster), to_yocto("300"), e.current_time(), to_yocto("120"), farm_id_0_booster_reward_claim, 0, 0, Some(FarmStatus::Running));
    assert_farm_detail(e.get_farm(&farm_id_1_booster), to_yocto("150"), e.current_time(), to_yocto("75"), farm_id_1_booster_reward_claim, to_yocto("15"), 0, Some(FarmStatus::Running));
    assert!(e.get_unclaimed_rewards(&users.farmer1, &seed_id_booster, &tokens.neth) <= to_yocto("15"));
    assert!(e.get_unclaimed_rewards(&users.farmer1, &seed_id_booster, &tokens.ndai) <= to_yocto("7.5"));
    assert!(e.get_unclaimed_rewards(&users.farmer2, &seed_id_booster, &tokens.neth) <= to_yocto("60"));
    assert!(e.get_unclaimed_rewards(&users.farmer2, &seed_id_booster, &tokens.ndai) <= to_yocto("30"));


    println!("normal> farmer2 force_unlock all seed with slashed at : {}, meanwhile claim all reward by seed", e.current_time());
    e.force_unlock(&users.farmer2, &seed_id_normal, to_yocto("50")).assert_success();
    let full_slashed = u128_ratio(to_yocto("50"), DEFAULT_SEED_SLASH_RATE as u128, BP_DENOM);
    let seed_slashed = u128_ratio(full_slashed, (farmer2_unlock_time - e.current_time()) as u128, to_nano(DEFAULT_MAX_LOCKING_DURATION_SEC) as u128);
    let free_amount = to_yocto("50") - seed_slashed;
    assert_user_seed_info(e.get_farmer_seed(&users.farmer2, &seed_id_normal), free_amount, 0, 0, 0, 0);
    assert_eq!(true, e.unlock_and_withdraw_seed(&users.farmer2, &seed_id_normal, 0, free_amount).unwrap_json::<bool>());
    assert_eq!(e.mft_balance_of(&users.farmer2, &token_id_normal), to_yocto("50") + free_amount);
    e.get_farmer_seed(&users.farmer2, &seed_id_normal).is_null();
    assert_seed(e.get_seed(&seed_id_normal), &seed_id_normal, TOKEN_DECIMALS as u32, 2, to_yocto("50"), to_yocto("100"), MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);

    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id_normal, &tokens.nref), 0);
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id_normal, &tokens.wnear), 0);
    assert_eq!(e.get_farmer_reward(&users.farmer2, &tokens.nref), to_yocto("60"));
    assert_eq!(e.get_farmer_reward(&users.farmer2, &tokens.wnear), to_yocto("30"));
    e.ft_storage_deposit(&users.farmer2, &tokens.nref);
    e.ft_storage_deposit(&users.farmer2, &tokens.wnear);
    assert_eq!(true, e.withdraw_reward(&users.farmer2, &tokens.nref, None).unwrap_json::<bool>());
    assert_eq!(true, e.withdraw_reward(&users.farmer2, &tokens.wnear, None).unwrap_json::<bool>());
    assert_eq!(e.get_farmer_reward(&users.farmer2, &tokens.nref), 0);
    assert_eq!(e.get_farmer_reward(&users.farmer2, &tokens.wnear), 0);
    assert_eq!(e.ft_balance_of(&tokens.nref, &users.farmer2), to_yocto("60"));
    assert_eq!(e.ft_balance_of(&tokens.wnear, &users.farmer2), to_yocto("30"));

    println!("booster> farmer2 force_unlock all seed with slashed at : {}, meanwhile claim all reward by seed", e.current_time());
    farm_id_0_booster_reward_claim += e.get_unclaimed_rewards(&users.farmer2, &seed_id_booster, &tokens.neth);
    farm_id_1_booster_reward_claim += e.get_unclaimed_rewards(&users.farmer2, &seed_id_booster, &tokens.ndai);
    e.force_unlock(&users.farmer2, &seed_id_booster, to_yocto("50")).assert_success();
    let full_slashed = u128_ratio(to_yocto("50"), DEFAULT_SEED_SLASH_RATE as u128, BP_DENOM);
    let seed_slashed = u128_ratio(full_slashed, (farmer2_unlock_time - e.current_time()) as u128, to_nano(DEFAULT_MAX_LOCKING_DURATION_SEC) as u128);
    let free_amount = to_yocto("50") - seed_slashed;
    assert_user_seed_info(e.get_farmer_seed(&users.farmer2, &seed_id_booster), free_amount, 0, 0, 0, 0);
    assert_eq!(true, e.unlock_and_withdraw_seed(&users.farmer2, &seed_id_booster, 0, free_amount).unwrap_json::<bool>());
    assert_eq!(e.mft_balance_of(&users.farmer2, &token_id_booster), to_yocto("50") + free_amount);
    e.get_farmer_seed(&users.farmer2, &seed_id_booster).is_null();
    assert_seed(e.get_seed(&seed_id_booster), &seed_id_booster, TOKEN_DECIMALS as u32, 2, to_yocto("50"), 200000000000000004764729344, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id_normal, &tokens.neth), 0);
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id_normal, &tokens.ndai), 0);
    assert!(e.get_farmer_reward(&users.farmer2, &tokens.neth) <= to_yocto("60"));
    assert!(e.get_farmer_reward(&users.farmer2, &tokens.ndai) <= to_yocto("30"));
    e.ft_storage_deposit(&users.farmer2, &tokens.neth);
    e.ft_storage_deposit(&users.farmer2, &tokens.ndai);
    assert_eq!(true, e.withdraw_reward(&users.farmer2, &tokens.neth, None).unwrap_json::<bool>());
    assert_eq!(true, e.withdraw_reward(&users.farmer2, &tokens.ndai, None).unwrap_json::<bool>());
    assert_eq!(e.get_farmer_reward(&users.farmer2, &tokens.neth), 0);
    assert_eq!(e.get_farmer_reward(&users.farmer2, &tokens.ndai), 0);
    assert!(e.ft_balance_of(&tokens.neth, &users.farmer2) <= to_yocto("60"));
    assert!(e.ft_balance_of(&tokens.ndai, &users.farmer2) <= to_yocto("30"));


    e.skip_time(DEFAULT_MAX_LOCKING_DURATION_SEC);
    println!(">> time pass {}, now at : {}", to_nano(DEFAULT_MAX_LOCKING_DURATION_SEC), e.current_time());
    assert_farm_detail(e.get_farm(&farm_id_0_normal), to_yocto("300"), e.current_time(), to_yocto("300"), to_yocto("105"), 0, 0, Some(FarmStatus::Ended));
    assert_farm_detail(e.get_farm(&farm_id_1_normal), to_yocto("150"), e.current_time(), to_yocto("150"), to_yocto("52.5"), to_yocto("15"), 0, Some(FarmStatus::Ended));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id_normal, &tokens.nref), to_yocto("195"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id_normal, &tokens.wnear), to_yocto("82.5"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id_normal, &tokens.nref), 0);
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id_normal, &tokens.wnear), 0);

    assert_farm_detail(e.get_farm(&farm_id_0_booster), to_yocto("300"), e.current_time(), to_yocto("300"), farm_id_0_booster_reward_claim, 0, 0, Some(FarmStatus::Ended));
    assert_farm_detail(e.get_farm(&farm_id_1_booster), to_yocto("150"), e.current_time(), to_yocto("150"), farm_id_1_booster_reward_claim, to_yocto("15"), 0, Some(FarmStatus::Ended));
    assert!(e.get_unclaimed_rewards(&users.farmer1, &seed_id_booster, &tokens.neth) <= to_yocto("195"));
    assert!(e.get_unclaimed_rewards(&users.farmer1, &seed_id_booster, &tokens.ndai) <= to_yocto("82.5"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id_booster, &tokens.neth), 0);
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id_booster, &tokens.ndai), 0);


    println!("normal> farmer1 unclock and withdraw all seed at : {}, meanwhile claim all reward by seed", e.current_time());
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id_normal), to_yocto("50"));
    assert_eq!(true, e.unlock_and_withdraw_seed(&users.farmer1, &seed_id_normal, to_yocto("50"), to_yocto("50")).unwrap_json::<bool>());
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id_normal), to_yocto("100"));
    assert_seed(e.get_seed(&seed_id_normal), &seed_id_normal, TOKEN_DECIMALS as u32, 2, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    assert_farm_detail(e.get_farm(&farm_id_0_normal), to_yocto("300"), e.current_time(), to_yocto("300"), to_yocto("300"), 0, 0, Some(FarmStatus::Ended));
    assert_farm_detail(e.get_farm(&farm_id_1_normal), to_yocto("150"), e.current_time(), to_yocto("150"), to_yocto("135"), to_yocto("15"), 0, Some(FarmStatus::Ended));
    assert!(e.get_farmer_seed(&users.farmer1, &seed_id_normal).is_null());

    println!("booster> farmer1 unclock and withdraw all seed at : {}, meanwhile claim all reward by seed", e.current_time());
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id_booster), to_yocto("50"));
    farm_id_0_booster_reward_claim += e.get_unclaimed_rewards(&users.farmer1, &seed_id_booster, &tokens.neth);
    farm_id_1_booster_reward_claim += e.get_unclaimed_rewards(&users.farmer1, &seed_id_booster, &tokens.ndai);
    assert_eq!(true, e.unlock_and_withdraw_seed(&users.farmer1, &seed_id_booster, to_yocto("50"), to_yocto("50")).unwrap_json::<bool>());
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id_booster), to_yocto("100"));
    assert_seed(e.get_seed(&seed_id_booster), &seed_id_booster, TOKEN_DECIMALS as u32, 2, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    assert_farm_detail(e.get_farm(&farm_id_0_booster), to_yocto("300"), e.current_time(), to_yocto("300"), farm_id_0_booster_reward_claim, 0, 0, Some(FarmStatus::Ended));
    assert_farm_detail(e.get_farm(&farm_id_1_booster), to_yocto("150"), e.current_time(), to_yocto("150"), farm_id_1_booster_reward_claim, to_yocto("15"), 0, Some(FarmStatus::Ended));
    assert!(e.get_farmer_seed(&users.farmer1, &seed_id_booster).is_null());


    println!("normal> restarted farm_id_0 at : {}", e.current_time());
    e.deposit_reward(&tokens.nref, &users.operator, to_yocto("75"), &farm_id_0_normal).assert_success();
    assert_farm_detail(e.get_farm(&farm_id_0_normal), to_yocto("375"), e.current_time(), to_yocto("300"), to_yocto("300"), 0, 0, Some(FarmStatus::Running));
    assert_farm_detail(e.get_farm(&farm_id_1_normal), to_yocto("150"), e.current_time(), to_yocto("150"), to_yocto("135"), to_yocto("15"), 0, Some(FarmStatus::Ended));

    println!("booster> restarted farm_id_0 at : {}", e.current_time());
    e.deposit_reward(&tokens.neth, &users.operator, to_yocto("75"), &farm_id_0_booster).assert_success();
    assert_farm_detail(e.get_farm(&farm_id_0_booster), to_yocto("375"), e.current_time(), to_yocto("300"), farm_id_0_booster_reward_claim, 0, 0, Some(FarmStatus::Running));
    assert_farm_detail(e.get_farm(&farm_id_1_booster), to_yocto("150"), e.current_time(), to_yocto("150"), farm_id_1_booster_reward_claim, to_yocto("15"), 0, Some(FarmStatus::Ended));


    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id_0_normal), to_yocto("375"), e.current_time(), to_yocto("330"), to_yocto("300"), to_yocto("30"), 0, Some(FarmStatus::Running));
    assert_farm_detail(e.get_farm(&farm_id_0_booster), to_yocto("375"), e.current_time(), to_yocto("330"), farm_id_0_booster_reward_claim, to_yocto("30"), 0, Some(FarmStatus::Running));


    println!("normal> farmer1 and farmer2 stake_free_seed at : {}", e.current_time());
    e.mft_stake_free_seed(&users.farmer1, &token_id_normal, to_yocto("100")).assert_success();
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id_normal), to_yocto("100"), 0, 0, 0, 0);
    assert_seed(e.get_seed(&seed_id_normal), &seed_id_normal, TOKEN_DECIMALS as u32, 2, to_yocto("100"), to_yocto("100"), MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    e.mft_stake_free_seed(&users.farmer2, &token_id_normal, to_yocto("50")).assert_success();
    assert_user_seed_info(e.get_farmer_seed(&users.farmer2, &seed_id_normal), to_yocto("50"), 0, 0, 0, 0);
    assert_seed(e.get_seed(&seed_id_normal), &seed_id_normal, TOKEN_DECIMALS as u32, 2, to_yocto("150"), to_yocto("150"), MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);

    println!("booster> farmer1 and farmer2 mft_stake_free_seed at : {}", e.current_time());
    e.mft_stake_free_seed(&users.farmer1, &token_id_booster, to_yocto("100")).assert_success();
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id_booster), to_yocto("100"), 0, 0, 0, 0);
    assert_seed(e.get_seed(&seed_id_booster), &seed_id_booster, TOKEN_DECIMALS as u32, 2, to_yocto("100"), 200000000000000004764729344, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    e.mft_stake_free_seed(&users.farmer2, &token_id_booster, to_yocto("50")).assert_success();
    assert_user_seed_info(e.get_farmer_seed(&users.farmer2, &seed_id_booster), to_yocto("50"), 0, 0, 0, 0);
    assert_seed(e.get_seed(&seed_id_booster), &seed_id_booster, TOKEN_DECIMALS as u32, 2, to_yocto("150"), 300000000000000007147094016, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    

    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id_0_normal), to_yocto("375"), e.current_time(), to_yocto("360"), to_yocto("300"), to_yocto("30"), 0, Some(FarmStatus::Running));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id_normal, &tokens.nref), to_yocto("20"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id_normal, &tokens.nref), to_yocto("10"));

    assert_farm_detail(e.get_farm(&farm_id_0_booster), to_yocto("375"), e.current_time(), to_yocto("360"), farm_id_0_booster_reward_claim, to_yocto("30"), 0, Some(FarmStatus::Running));
    assert!(e.get_unclaimed_rewards(&users.farmer1, &seed_id_booster, &tokens.neth) <= to_yocto("20"));
    assert!(e.get_unclaimed_rewards(&users.farmer2, &seed_id_booster, &tokens.neth) <= to_yocto("10"));

    println!("booster> farmer1 withdraw all booster seed at : {}", e.current_time());
    assert_eq!(e.ft_balance_of(&tokens.love_ref, &users.farmer1), 0);
    println!("{}", e.get_farmer_seed(&users.farmer1, &booster_id));
    farm_id_0_booster_reward_claim += e.get_unclaimed_rewards(&users.farmer1, &seed_id_booster, &tokens.neth);
    assert_eq!(true, e.unlock_and_withdraw_seed(&users.farmer1, &booster_id, 0, to_yocto("10")).unwrap_json::<bool>());
    assert_eq!(e.ft_balance_of(&tokens.love_ref, &users.farmer1), to_yocto("10"));
    assert_eq!(e.get_farmer_seed(&users.farmer1, &seed_id_booster).get("boost_ratios").unwrap()[booster_id.clone()], Value::Null);
    assert_seed(e.get_seed(&seed_id_booster), &seed_id_booster, TOKEN_DECIMALS as u32, 2, to_yocto("150"), 200000000000000002382364672, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);

    println!("booster> farmer2 withdraw all booster seed at : {}", e.current_time());
    assert_eq!(e.ft_balance_of(&tokens.love_ref, &users.farmer2), 0);
    println!("{}", e.get_farmer_seed(&users.farmer2, &booster_id));
    farm_id_0_booster_reward_claim += e.get_unclaimed_rewards(&users.farmer2, &seed_id_booster, &tokens.neth);
    assert_eq!(true, e.unlock_and_withdraw_seed(&users.farmer2, &booster_id, 0, to_yocto("9")).unwrap_json::<bool>());
    assert_eq!(e.ft_balance_of(&tokens.love_ref, &users.farmer2), to_yocto("9"));
    assert_eq!(e.get_farmer_seed(&users.farmer2, &seed_id_booster).get("boost_ratios").unwrap()[booster_id.clone()], json!(0.0));
    assert_seed(e.get_seed(&seed_id_booster), &seed_id_booster, TOKEN_DECIMALS as u32, 2, to_yocto("150"), to_yocto("150"), MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    assert_eq!(true, e.unlock_and_withdraw_seed(&users.farmer2, &booster_id, 0, to_yocto("1")).unwrap_json::<bool>());
    assert_eq!(e.ft_balance_of(&tokens.love_ref, &users.farmer2), to_yocto("10"));
    assert_eq!(e.get_farmer_seed(&users.farmer2, &seed_id_booster).get("boost_ratios").unwrap()[booster_id.clone()], Value::Null);
    assert_seed(e.get_seed(&seed_id_booster), &seed_id_booster, TOKEN_DECIMALS as u32, 2, to_yocto("150"), to_yocto("150"), MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);

    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}, Insufficient reward, send all", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id_0_normal), to_yocto("375"), e.current_time(), to_yocto("375"), to_yocto("300"), to_yocto("30"), 0, Some(FarmStatus::Ended));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id_normal, &tokens.nref), to_yocto("30"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id_normal, &tokens.nref), to_yocto("15"));

    assert_farm_detail(e.get_farm(&farm_id_0_booster), to_yocto("375"), e.current_time(), to_yocto("375"), farm_id_0_booster_reward_claim, to_yocto("30"), 0, Some(FarmStatus::Ended));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id_booster, &tokens.neth), to_yocto("10"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id_booster, &tokens.neth), to_yocto("5"));
}

#[test]
fn test_booster_seed_mutli_farm(){
    let e = init_env();
    let users = Users::init(&e);
    let tokens = Tokens::init(&e);

    let seed_id = tokens.love_ref.account_id.to_string();
    let farm_id_0 = format!("{}#{}", seed_id, 0);
    let farm_id_1 = format!("{}#{}", seed_id, 1);

    println!("> create_seed at : {}", e.current_time());
    e.create_seed(&e.owner, &seed_id, TOKEN_DECIMALS as u32, None, Some(0)).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, 0);


    println!("> farmer1 mint ft at : {}", e.current_time());
    e.ft_mint(&tokens.love_ref, &users.farmer1, to_yocto("100"));
    assert_eq!(e.ft_balance_of(&tokens.love_ref, &users.farmer1), to_yocto("100"));

    println!("> farmer1 register farming at : {}", e.current_time());
    e.storage_deposit_self_to_farming(&users.farmer1).assert_success();

    println!("> farmer2 mint ft at : {}", e.current_time());
    e.ft_mint(&tokens.love_ref, &users.farmer2, to_yocto("100"));
    assert_eq!(e.ft_balance_of(&tokens.love_ref, &users.farmer2), to_yocto("100"));

    println!("> farmer2 register farming at : {}", e.current_time());
    e.storage_deposit_self_to_farming(&users.farmer2).assert_success();

    println!("> farming register mft at : {}", e.current_time());
    e.ft_storage_deposit(&users.farmer2, &tokens.love_ref);


    let start_at = e.current_time();
    println!("> create_farm at : {}", e.current_time());
    e.create_farm(&e.owner, &seed_id, &tokens.nref, to_sec(start_at + NANOS_PER_DAY), to_yocto("30")).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 1, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, 0);
    assert_farm_info(e.get_farm(&farm_id_0), &farm_id_0, &tokens.nref, to_sec(start_at + NANOS_PER_DAY), to_yocto("30"));
    assert_farm_detail(e.get_farm(&farm_id_0), 0, start_at + NANOS_PER_DAY, 0, 0, 0, 0, Some(FarmStatus::Created));

    e.create_farm(&e.owner, &seed_id, &tokens.wnear, to_sec(start_at), to_yocto("15")).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 2, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, 0);
    assert_farm_info(e.get_farm(&farm_id_1), &farm_id_1, &tokens.wnear, to_sec(start_at), to_yocto("15"));
    assert_farm_detail(e.get_farm(&farm_id_1), 0, start_at, 0, 0, 0, 0, Some(FarmStatus::Created));


    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id_0), 0, e.current_time(), 0, 0, 0, 0, Some(FarmStatus::Created));
    assert_farm_detail(e.get_farm(&farm_id_1), 0, e.current_time() - NANOS_PER_DAY, 0, 0, 0, 0, Some(FarmStatus::Pending));

    
    println!("> deposit_reward at : {}", e.current_time());
    e.ft_mint(&tokens.nref, &users.operator, to_yocto("10000"));
    assert_eq!(e.ft_balance_of(&tokens.nref, &users.operator), to_yocto("10000"));
    e.deposit_reward(&tokens.nref, &users.operator, to_yocto("300"), &farm_id_0).assert_success();
    assert_farm_detail(e.get_farm(&farm_id_0), to_yocto("300"), e.current_time(), 0, 0, 0, 0, Some(FarmStatus::Created));
    
    e.ft_mint(&tokens.wnear, &users.operator, to_yocto("10000"));
    assert_eq!(e.ft_balance_of(&tokens.wnear, &users.operator), to_yocto("10000"));
    e.deposit_reward(&tokens.wnear, &users.operator, to_yocto("150"), &farm_id_1).assert_success();
    assert_farm_detail(e.get_farm(&farm_id_1), to_yocto("150"), e.current_time(), to_yocto("15"), 0, to_yocto("15"), 0, Some(FarmStatus::Running));
    

    println!("> farmer1 stake_free_seed at : {}", e.current_time());
    e.ft_stake_free_seed(&users.farmer1, &tokens.love_ref, to_yocto("100")).assert_success();
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id), to_yocto("100"), 0, 0, 0, 0);
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 2, to_yocto("100"), to_yocto("100"), MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, 0);


    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id_0), to_yocto("300"), e.current_time(), to_yocto("30"), 0, 0, 0, Some(FarmStatus::Running));
    assert_farm_detail(e.get_farm(&farm_id_1), to_yocto("150"), e.current_time(), to_yocto("30"), 0, to_yocto("15"), 0, Some(FarmStatus::Running));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref), to_yocto("30"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.wnear), to_yocto("15"));


    println!("> farmer2 stake_free_seed at : {}", e.current_time());
    e.ft_stake_free_seed(&users.farmer2, &tokens.love_ref, to_yocto("100")).assert_success();
    assert_user_seed_info(e.get_farmer_seed(&users.farmer2, &seed_id), to_yocto("100"), 0, 0, 0, 0);
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 2, to_yocto("200"), to_yocto("200"), MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, 0);


    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id_0), to_yocto("300"), e.current_time(), to_yocto("60"), 0, 0, 0, Some(FarmStatus::Running));
    assert_farm_detail(e.get_farm(&farm_id_1), to_yocto("150"), e.current_time(), to_yocto("45"), 0, to_yocto("15"), 0, Some(FarmStatus::Running));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref), to_yocto("45"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.wnear), to_yocto("22.5"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.nref), to_yocto("15"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.wnear), to_yocto("7.5"));


    println!("> farmer1 claim_reward_by_seed at : {}", e.current_time());
    assert_eq!(e.get_farmer_reward(&users.farmer1, &tokens.nref), 0);
    assert_eq!(e.get_farmer_reward(&users.farmer1, &tokens.wnear), 0);
    e.claim_reward_by_seed(&users.farmer1, &seed_id).assert_success();
    assert_eq!(e.get_farmer_reward(&users.farmer1, &tokens.nref), to_yocto("45"));
    assert_eq!(e.get_farmer_reward(&users.farmer1, &tokens.wnear), to_yocto("22.5"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref), 0);
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.wnear), 0);
    assert_farm_detail(e.get_farm(&farm_id_0), to_yocto("300"), e.current_time(), to_yocto("60"), to_yocto("45"), 0, 0, Some(FarmStatus::Running));
    assert_farm_detail(e.get_farm(&farm_id_1), to_yocto("150"), e.current_time(), to_yocto("45"), to_yocto("22.5"), to_yocto("15"), 0, Some(FarmStatus::Running));


    println!("> farmer1 withdraw all seed at : {}", e.current_time());
    assert_eq!(e.ft_balance_of(&tokens.love_ref, &users.farmer1), 0);
    assert_eq!(true, e.unlock_and_withdraw_seed(&users.farmer1, &seed_id, 0, to_yocto("50")).unwrap_json::<bool>());
    assert_eq!(e.ft_balance_of(&tokens.love_ref, &users.farmer1), to_yocto("50"));
    assert_eq!(true, e.unlock_and_withdraw_seed(&users.farmer1, &seed_id, 0, to_yocto("50")).unwrap_json::<bool>());
    assert_eq!(e.ft_balance_of(&tokens.love_ref, &users.farmer1), to_yocto("100"));
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 2, to_yocto("100"), to_yocto("100"), MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, 0);


    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id_0), to_yocto("300"), e.current_time(), to_yocto("90"), to_yocto("45"), 0, 0, Some(FarmStatus::Running));
    assert_farm_detail(e.get_farm(&farm_id_1), to_yocto("150"), e.current_time(), to_yocto("60"), to_yocto("22.5"), to_yocto("15"), 0, Some(FarmStatus::Running));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref), 0);
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.wnear), 0);
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.nref), to_yocto("45"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.wnear), to_yocto("22.5"));


    println!("> farmer1 stake_free_seed again at : {}", e.current_time());
    e.ft_stake_free_seed(&users.farmer1, &tokens.love_ref, to_yocto("50")).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 2, to_yocto("150"), to_yocto("150"), MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, 0);
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id), to_yocto("50"), 0, 0, 0, 0);


    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id_0), to_yocto("300"), e.current_time(), to_yocto("120"), to_yocto("45"), 0, 0, Some(FarmStatus::Running));
    assert_farm_detail(e.get_farm(&farm_id_1), to_yocto("150"), e.current_time(), to_yocto("75"), to_yocto("22.5"), to_yocto("15"), 0, Some(FarmStatus::Running));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref), to_yocto("10"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.wnear), to_yocto("5"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.nref), to_yocto("65"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.wnear), to_yocto("32.5"));


    println!("> farmer2 withdraw all seed at : {}, meanwhile claim all reward by seed", e.current_time());
    assert_eq!(true, e.unlock_and_withdraw_seed(&users.farmer2, &seed_id, 0, to_yocto("100")).unwrap_json::<bool>());
    assert_eq!(e.ft_balance_of(&tokens.love_ref, &users.farmer2), to_yocto("100"));
    e.get_farmer_seed(&users.farmer2, &seed_id).is_null();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 2, to_yocto("50"), to_yocto("50"), MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, 0);
    
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.nref), 0);
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.wnear), 0);
    assert_eq!(e.get_farmer_reward(&users.farmer2, &tokens.nref), to_yocto("65"));
    assert_eq!(e.get_farmer_reward(&users.farmer2, &tokens.wnear), to_yocto("32.5"));
    e.ft_storage_deposit(&users.farmer2, &tokens.nref);
    e.ft_storage_deposit(&users.farmer2, &tokens.wnear);
    assert_eq!(true, e.withdraw_reward(&users.farmer2, &tokens.nref, None).unwrap_json::<bool>());
    assert_eq!(true, e.withdraw_reward(&users.farmer2, &tokens.wnear, None).unwrap_json::<bool>());
    assert_eq!(e.get_farmer_reward(&users.farmer2, &tokens.nref), 0);
    assert_eq!(e.get_farmer_reward(&users.farmer2, &tokens.wnear), 0);
    assert_eq!(e.ft_balance_of(&tokens.nref, &users.farmer2), to_yocto("65"));
    assert_eq!(e.ft_balance_of(&tokens.wnear, &users.farmer2), to_yocto("32.5"));


    e.skip_time(DEFAULT_MAX_LOCKING_DURATION_SEC);
    println!(">> time pass {}, now at : {}", to_nano(DEFAULT_MAX_LOCKING_DURATION_SEC), e.current_time());
    assert_farm_detail(e.get_farm(&farm_id_0), to_yocto("300"), e.current_time(), to_yocto("300"), to_yocto("110"), 0, 0, Some(FarmStatus::Ended));
    assert_farm_detail(e.get_farm(&farm_id_1), to_yocto("150"), e.current_time(), to_yocto("150"), to_yocto("55"), to_yocto("15"), 0, Some(FarmStatus::Ended));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref), to_yocto("190"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.wnear), to_yocto("80"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.nref), 0);
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.wnear), 0);


    println!("> farmer1 unclock and withdraw all seed at : {}, meanwhile claim all reward by seed", e.current_time());
    assert_eq!(e.ft_balance_of(&tokens.love_ref, &users.farmer1), to_yocto("50"));
    assert_eq!(true, e.unlock_and_withdraw_seed(&users.farmer1, &seed_id, 0, to_yocto("50")).unwrap_json::<bool>());
    assert_eq!(e.ft_balance_of(&tokens.love_ref, &users.farmer1), to_yocto("100"));
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 2, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, 0);
    assert_farm_detail(e.get_farm(&farm_id_0), to_yocto("300"), e.current_time(), to_yocto("300"), to_yocto("300"), 0, 0, Some(FarmStatus::Ended));
    assert_farm_detail(e.get_farm(&farm_id_1), to_yocto("150"), e.current_time(), to_yocto("150"), to_yocto("135"), to_yocto("15"), 0, Some(FarmStatus::Ended));


    println!("> restarted farm_id_0 at : {}", e.current_time());
    e.deposit_reward(&tokens.nref, &users.operator, to_yocto("75"), &farm_id_0).assert_success();
    assert_farm_detail(e.get_farm(&farm_id_0), to_yocto("375"), e.current_time(), to_yocto("300"), to_yocto("300"), 0, 0, Some(FarmStatus::Running));
    assert_farm_detail(e.get_farm(&farm_id_1), to_yocto("150"), e.current_time(), to_yocto("150"), to_yocto("135"), to_yocto("15"), 0, Some(FarmStatus::Ended));


    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id_0), to_yocto("375"), e.current_time(), to_yocto("330"), to_yocto("300"), to_yocto("30"), 0, Some(FarmStatus::Running));
    assert_farm_detail(e.get_farm(&farm_id_1), to_yocto("150"), e.current_time(), to_yocto("150"), to_yocto("135"), to_yocto("15"), 0, Some(FarmStatus::Ended));


    println!("> farmer1 and farmer2 stake_free_seed at : {}", e.current_time());
    e.ft_stake_free_seed(&users.farmer1, &tokens.love_ref, to_yocto("100")).assert_success();
    assert_user_seed_info(e.get_farmer_seed(&users.farmer1, &seed_id), to_yocto("100"), 0, 0, 0, 0);
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 2, to_yocto("100"), to_yocto("100"), MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, 0);
    e.ft_stake_free_seed(&users.farmer2, &tokens.love_ref, to_yocto("50")).assert_success();
    assert_user_seed_info(e.get_farmer_seed(&users.farmer2, &seed_id), to_yocto("50"), 0, 0, 0, 0);
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 2, to_yocto("150"), to_yocto("150"), MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, 0);


    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id_0), to_yocto("375"), e.current_time(), to_yocto("360"), to_yocto("300"), to_yocto("30"), 0, Some(FarmStatus::Running));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref), to_yocto("20"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.nref), to_yocto("10"));

    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}, Insufficient reward, send all", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id_0), to_yocto("375"), e.current_time(), to_yocto("375"), to_yocto("300"), to_yocto("30"), 0, Some(FarmStatus::Ended));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref), to_yocto("30"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.nref), to_yocto("15"));

    assert_eq!(e.get_metadata().farm_count.0, 2);
    e.remove_farm_from_seed(&e.owner, &farm_id_0).assert_success();
    e.remove_farm_from_seed(&e.owner, &farm_id_1).assert_success();
    assert_eq!(e.get_metadata().farm_count.0, 0);
    assert_eq!(e.get_metadata().outdated_farm_count.0, 2);
}



#[test]
fn test_verify_users_reward_total_amount(){
    let e = init_env();
    let users = Users::init(&e);
    let tokens = Tokens::init(&e);

    let inner_id = "0".to_string();
    let token_id = format!(":{}", inner_id);
    let seed_id = e.mft_seed_id(&inner_id);
    let farm_id = format!("{}#{}", seed_id, 0);

    println!("> create_seed at : {}", e.current_time());
    e.create_seed(&e.owner, &seed_id, TOKEN_DECIMALS as u32, None, None).assert_success();

    println!("> farmer1 mint mft and register farming at : {}", e.current_time());
    e.mft_mint(&inner_id, &users.farmer1, to_yocto("100"));
    e.storage_deposit_self_to_farming(&users.farmer1).assert_success();

    println!("> farmer2 mint mft and register farming at : {}", e.current_time());
    e.mft_mint(&inner_id, &users.farmer2, to_yocto("100"));
    e.storage_deposit_self_to_farming(&users.farmer2).assert_success();

    println!("> bob mint mft and register farming at : {}", e.current_time());
    e.mft_mint(&inner_id, &users.bob, to_yocto("100"));
    e.storage_deposit_self_to_farming(&users.bob).assert_success();

    println!("> farming register mft at : {}", e.current_time());
    e.mft_storage_deposit(&token_id, &e.farming_contract.user_account);

    println!("> create_farm at : {}", e.current_time());
    e.create_farm(&e.owner, &seed_id, &tokens.nref, to_sec(e.current_time()), to_yocto("2")).assert_success();

    println!("> deposit_reward at : {}", e.current_time());
    e.ft_mint(&tokens.nref, &users.operator, to_yocto("10000"));
    e.deposit_reward(&tokens.nref, &users.operator, to_yocto("100"), &farm_id).assert_success();

    println!("> farmer1、 farmer2 mft_stake_free_seed at : {}", e.current_time());
    e.mft_stake_free_seed(&users.farmer1, &token_id, to_yocto("1.3")).assert_success();
    e.mft_stake_free_seed(&users.farmer2, &token_id, to_yocto("1.3")).assert_success();
    e.mft_stake_free_seed(&users.bob, &token_id, to_yocto("1.3")).assert_success();

    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id), to_yocto("100"), e.current_time(), to_yocto("2"), 0, 0, 0, Some(FarmStatus::Running));


    println!("{}", e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref));
    println!("{}", e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.nref));
    println!("{}", e.get_unclaimed_rewards(&users.bob, &seed_id, &tokens.nref));
    println!("{}", to_yocto("2"));
    println!("{}", e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref) +
    e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.nref) +
    e.get_unclaimed_rewards(&users.bob, &seed_id, &tokens.nref));
    assert!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref) +
         e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.nref) +
         e.get_unclaimed_rewards(&users.bob, &seed_id, &tokens.nref) <= to_yocto("2"));
}

#[test]
fn test_booster_ratio(){
    let e = init_env();
    let users = Users::init(&e);
    let tokens = Tokens::init(&e);

    let inner_id_booster = "0".to_string();
    let token_id_booster = format!(":{}", inner_id_booster);
    let seed_id_booster = e.mft_seed_id(&inner_id_booster);

    println!("> create seed affected by booster seed at : {}", e.current_time());
    e.create_seed(&e.owner, &seed_id_booster, TOKEN_DECIMALS as u32, None, None).assert_success();
    assert_seed(e.get_seed(&seed_id_booster), &seed_id_booster, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);

    println!("booster> create booster seed at : {}, min_locking_duration_sec is 0, can not lock", e.current_time());
    let booster_id = tokens.love_ref.account_id.to_string();
    let mut affected_seeds = HashMap::new();
    affected_seeds.insert(seed_id_booster.clone(), 10);
    let booster_info = BoosterInfo { booster_decimal: 18, affected_seeds: affected_seeds.clone()};
    e.create_seed(&e.owner, &booster_id, TOKEN_DECIMALS as u32, None, Some(0)).assert_success();
    assert_seed(e.get_seed(&booster_id), &booster_id, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, 0);
    e.modify_booster(&e.owner, &booster_id, &booster_info).assert_success();

    println!("> farmer1 mint mft at : {}", e.current_time());
    e.mft_mint(&inner_id_booster, &users.farmer1, to_yocto("100"));
    assert_eq!(e.mft_balance_of(&users.farmer1, &token_id_booster), to_yocto("100"));

    println!("> farmer1 register farming at : {}", e.current_time());
    e.storage_deposit_self_to_farming(&users.farmer1).assert_success();

    println!("> farming register mft at : {}", e.current_time());
    e.mft_storage_deposit(&token_id_booster, &e.farming_contract.user_account);

    e.mft_stake_free_seed(&users.farmer1, &token_id_booster, to_yocto("100")).assert_success();
    println!("booster> farmer1 stake booster token at : {}", e.current_time());
    e.ft_mint(&tokens.love_ref, &users.farmer1, to_yocto("10"));
    e.modify_seed_min_deposit(&e.owner, &booster_id, 0, 1).assert_success();
    e.ft_stake_free_seed(&users.farmer1, &tokens.love_ref, 5u128 * 10u128.pow(17)).assert_success();
    assert_eq!(e.get_farmer_seed(&users.farmer1, &seed_id_booster).get("boost_ratios").unwrap()[booster_id.clone()], json!(0.0));
    e.ft_stake_free_seed(&users.farmer1, &tokens.love_ref, 5u128 * 10u128.pow(17)).assert_success();
    assert_eq!(e.get_farmer_seed(&users.farmer1, &seed_id_booster).get("boost_ratios").unwrap()[booster_id.clone()], json!(0.0));
    e.ft_stake_free_seed(&users.farmer1, &tokens.love_ref, 1u128 * 10u128.pow(2)).assert_success();
    assert_eq!(e.get_farmer_seed(&users.farmer1, &seed_id_booster).get("boost_ratios").unwrap()[booster_id.clone()], json!(9.643274665532871e-17));
}


#[test]
fn test_reward_after_start_at(){
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

    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    println!("> no famer stake, first day reward to beneficiary");
    assert_farm_detail(e.get_farm(&farm_id), 0, start_at, 0, 0, 0, 0, Some(FarmStatus::Pending));

    println!("> farmer1 mft_stake_free_seed at : {}", e.current_time());
    e.mft_stake_free_seed(&users.farmer1, &token_id, to_yocto("100")).assert_success();

    e.skip_time(to_sec(2 * NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref), 0);

    println!("> farmer2 mft_stake_free_seed at : {}", e.current_time());
    e.mft_stake_free_seed(&users.farmer2, &token_id, to_yocto("100")).assert_success();

    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id), 0, start_at, 0, 0, 0, 0, Some(FarmStatus::Pending));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref), 0);
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.nref), 0);

    println!("> deposit_reward at : {}", e.current_time());
    e.ft_mint(&tokens.nref, &users.operator, to_yocto("10000"));
    assert_eq!(e.ft_balance_of(&tokens.nref, &users.operator), to_yocto("10000"));
    e.deposit_reward(&tokens.nref, &users.operator, to_yocto("100"), &farm_id).assert_success();
    assert_farm_detail(e.get_farm(&farm_id), to_yocto("100"), start_at + 4 * NANOS_PER_DAY, to_yocto("40"), 0, 0, 0, Some(FarmStatus::Running));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref), to_yocto("20"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.nref), to_yocto("20"));

    e.skip_time(to_sec(20 * NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    assert_farm_detail(e.get_farm(&farm_id), to_yocto("100"), start_at + 24 * NANOS_PER_DAY, to_yocto("100"), 0, 0, 0, Some(FarmStatus::Ended));

    e.deposit_reward(&tokens.nref, &users.operator, to_yocto("100"), &farm_id).assert_success();
    assert_eq!(e.get_unclaimed_rewards(&users.farmer1, &seed_id, &tokens.nref), to_yocto("50"));
    assert_eq!(e.get_unclaimed_rewards(&users.farmer2, &seed_id, &tokens.nref), to_yocto("50"));
}