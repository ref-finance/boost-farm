mod setup;
use crate::setup::*;

#[test]
fn test_update(){
    let e = Env::init_with_contract(previous_boost_farm_wasm_bytes());
    let users = Users::init(&e);

    assert_eq!(e.get_metadata020().version, "0.2.5".to_string());

    assert_err!(
        e.upgrade_contract(&users.alice, boost_farm_wasm_bytes()),
        E002_NOT_ALLOWED
    );

    e.upgrade_contract(&e.owner, boost_farm_wasm_bytes()).assert_success();
    assert_eq!(e.get_metadata().version, "0.3.0".to_string());
}