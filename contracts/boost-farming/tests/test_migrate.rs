mod setup;
use crate::setup::*;

#[test]
fn test_update(){
    let e = Env::init_with_contract(previous_boost_farm_wasm_bytes());
    let users = Users::init(&e);

    assert_eq!(e.get_metadata020().version, "0.3.2".to_string());

    let inner_id = "0".to_string();
    let token_id = format!(":{}", inner_id);
    let seed_id = e.mft_seed_id(&inner_id);

    println!("> create seed affected by booster seed at : {}", e.current_time());
    e.create_seed(&e.owner, &seed_id, TOKEN_DECIMALS as u32, None, None).assert_success();
    assert_seed(e.get_seed(&seed_id), &seed_id, TOKEN_DECIMALS as u32, 0, 0, 0, MIN_SEED_DEPOSIT, DEFAULT_SEED_SLASH_RATE, DEFAULT_SEED_MIN_LOCKING_DURATION_SEC);
    e.mft_mint(&inner_id, &users.farmer1, to_yocto("100"));
    e.mft_mint(&inner_id, &users.farmer2, to_yocto("60"));
    e.storage_deposit_self_to_farming(&users.farmer1).assert_success();
    e.storage_deposit_self_to_farming(&users.farmer2).assert_success();
    e.mft_storage_deposit(&token_id, &e.farming_contract.user_account);
    e.mft_stake_free_seed(&users.farmer1, &token_id, to_yocto("100")).assert_success();
    e.mft_stake_free_seed(&users.farmer2, &token_id, to_yocto("50")).assert_success();

    assert_err!(
        e.upgrade_contract(&users.alice, boost_farm_wasm_bytes()),
        E002_NOT_ALLOWED
    );

    e.upgrade_contract(&e.owner, boost_farm_wasm_bytes()).assert_success();
    assert_eq!(e.get_metadata().version, "0.4.0".to_string());

    println!("{:?}", e.get_farmer_seed(&users.farmer1, &seed_id));
    e.mft_stake_free_seed(&users.farmer2, &token_id, to_yocto("10")).assert_success();
    println!("{:?}", e.get_farmer_seed(&users.farmer2, &seed_id));
}