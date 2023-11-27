mod setup;
use crate::setup::*;

#[test]
fn test_storage(){
    let e = init_env();
    let users = Users::init(&e);
    assert_eq!(e.get_metadata().farmer_count.0, 0);

    // first register by self
    let user_balance_before = users.alice.account().unwrap().amount;
    e.storage_deposit(&users.alice, &users.alice, to_yocto("0.1")).assert_success();
    assert_eq!(e.get_metadata().farmer_count.0, 1);
    let user_balance_after = users.alice.account().unwrap().amount;
    assert!(user_balance_before - user_balance_after > to_yocto("0.1"));
    assert!(user_balance_before - user_balance_after < to_yocto("0.11"));

    // repeat register
    let user_balance_before = users.alice.account().unwrap().amount;
    e.storage_deposit(&users.alice, &users.alice, to_yocto("0.1")).assert_success();
    assert_eq!(e.get_metadata().farmer_count.0, 1);
    assert!(user_balance_before - user_balance_after < to_yocto("0.001"));

    // first register by other
    e.storage_deposit(&users.alice, &users.bob, to_yocto("0.1")).assert_success();
    assert_eq!(e.get_metadata().farmer_count.0, 2);

    // unregister
    let user_balance_before = users.alice.account().unwrap().amount;
    assert_eq!(e.storage_unregister(&users.alice, 1).unwrap_json::<bool>(), true);
    let user_balance_after = users.alice.account().unwrap().amount;
    assert!(user_balance_after - user_balance_before > to_yocto("0.09"));
    assert!(user_balance_after - user_balance_before < to_yocto("0.1"));
    assert_eq!(e.get_metadata().farmer_count.0, 1);

    // repeat unregister
    assert_eq!(e.storage_unregister(&users.alice, 1).unwrap_json::<bool>(), false);


    // error scene 
    // 1 : storage_withdraw with Requires attached deposit of exactly 1 yoctoNEAR
    assert_err!(
        e.storage_withdraw(&users.alice, 0),
        "Requires attached deposit of exactly 1 yoctoNEAR"
    );

    // 2 : storage_withdraw with E101_INSUFFICIENT_BALANCE
    assert_err!(
        e.storage_withdraw(&users.alice, 1),
        E006_NOT_IMPLEMENTED
    );

    // 3 : storage_unregister with storage_withdraw with Requires attached deposit of exactly 1 yoctoNEAR
    assert_err!(
        e.storage_unregister(&users.alice, 0),
        "Requires attached deposit of exactly 1 yoctoNEAR"
    );

    let tokens = Tokens::init(&e);
    let inner_id = "0".to_string();
    let token_id = format!(":{}", inner_id);
    let seed_id = e.mft_seed_id(&inner_id);
    let farm_id = format!("{}#{}", seed_id, 0);

    println!("> create_seed at : {}", e.current_time());
    e.create_seed(&e.owner, &seed_id, TOKEN_DECIMALS as u32, None, None).assert_success();
    println!("> create_farm at : {}", e.current_time());
    e.create_farm(&e.owner, &seed_id, &tokens.nref, to_sec(e.current_time()), to_yocto("10")).assert_success();
    println!("> deposit_reward at : {}", e.current_time());
    e.ft_mint(&tokens.nref, &users.operator, to_yocto("10000"));
    e.deposit_reward(&tokens.nref, &users.operator, to_yocto("100"), &farm_id).assert_success();

    println!("> farmer1 mft_stake_free_seed at : {}", e.current_time());
    e.mft_mint(&inner_id, &users.bob, to_yocto("100"));
    e.mft_storage_deposit(&token_id, &e.farming_contract.user_account);
    e.mft_stake_free_seed(&users.bob, &token_id, to_yocto("100")).assert_success();
    println!("{:?}", e.get_farmer_seed(&users.bob, &seed_id));

    // 4 : storage_unregister with E104_STILL_HAS_SEED
    assert_err!(
        e.storage_unregister(&users.bob, 1),
        E104_STILL_HAS_SEED
    );

    e.skip_time(to_sec(NANOS_PER_DAY));
    println!(">> time pass {}, now at : {}", NANOS_PER_DAY, e.current_time());
    e.claim_reward_by_seed(&users.bob, &seed_id).assert_success();

    // 5 : storage_unregister with E103_STILL_HAS_REWARD
    assert_err!(
        e.storage_unregister(&users.bob, 1),
        E103_STILL_HAS_REWARD
    );
}