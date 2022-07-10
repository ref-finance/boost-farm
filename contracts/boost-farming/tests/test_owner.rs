mod setup;
use crate::setup::*;


#[test]
fn test_set_owner(){
    let e = init_env();
    let users = Users::init(&e);
    assert_eq!(e.get_metadata().owner_id, e.owner.account_id());
    
    // error scene 
    // 1 : Requires attached deposit of exactly 1 yoctoNEAR
    assert_err!(
        e.set_owner(&users.alice, &e.owner, 0),
        "Requires attached deposit of exactly 1 yoctoNEAR"
    );

    // 2: E002_NOT_ALLOWED
    assert_err!(
        e.set_owner(&users.alice, &e.owner, 1),
        E002_NOT_ALLOWED
    );

    // success
    e.set_owner(&e.owner, &users.alice, 1).assert_success();
    assert_eq!(e.get_metadata().owner_id, users.alice.account_id());
}

#[test]
fn test_pause_resume(){
    let e = init_env();
    let users = Users::init(&e);
    assert_eq!(e.get_metadata().state, RunningState::Running);
    
    // error scene 
    // 1 : Requires attached deposit of exactly 1 yoctoNEAR
    assert_err!(
        e.pause_contract(&users.alice, 0),
        "Requires attached deposit of exactly 1 yoctoNEAR"
    );

    // 2: E002_NOT_ALLOWED
    assert_err!(
        e.pause_contract(&users.alice, 1),
        E002_NOT_ALLOWED
    );

    // success
    e.pause_contract(&e.owner, 1).assert_success();
    assert_eq!(e.get_metadata().state, RunningState::Paused);

    let inner_id = "0".to_string();
    let seed_id = e.mft_seed_id(&inner_id);
    assert_err!(
        e.create_seed(&e.owner, &seed_id, TOKEN_DECIMALS as u32, None, None),
        E004_CONTRACT_PAUSED
    );

    // success
    e.resume_contract(&e.owner, 1).assert_success();
    assert_eq!(e.get_metadata().state, RunningState::Running);

    e.create_seed(&e.owner, &seed_id, TOKEN_DECIMALS as u32, None, None).assert_success();
}

#[test]
fn test_operators(){
    let e = init_env();
    let users = Users::init(&e);

    // error scene 
    // 1 : extend with Requires attached deposit of exactly 1 yoctoNEAR
    assert_err!(
        e.extend_operators(&users.alice, vec![], 0),
        "Requires attached deposit of exactly 1 yoctoNEAR"
    );

    // 2 : extend with E002_NOT_ALLOWED
    assert_err!(
        e.extend_operators(&users.alice, vec![], 1),
        E002_NOT_ALLOWED
    );

    // 3 : remove with Requires attached deposit of exactly 1 yoctoNEAR
    assert_err!(
        e.remove_operators(&users.alice, vec![], 0),
        "Requires attached deposit of exactly 1 yoctoNEAR"
    );

    // 4 : remove with E002_NOT_ALLOWED
    assert_err!(
        e.remove_operators(&users.alice, vec![], 1),
        E002_NOT_ALLOWED
    );

    // success
    assert_eq!(e.get_metadata().operators, vec![]);
    e.set_owner(&e.owner, &users.alice, 1).assert_success();
    e.extend_operators(&users.alice, vec![&users.bob], 1).assert_success();
    assert_eq!(e.get_metadata().operators, vec![users.bob.account_id()]);
    e.remove_operators(&users.alice, vec![&users.bob], 1).assert_success();
    assert_eq!(e.get_metadata().operators, vec![]);
}
