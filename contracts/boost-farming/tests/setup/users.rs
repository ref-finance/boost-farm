
use crate::*;
pub use near_sdk_sim::UserAccount;


pub struct Users {
    pub alice: UserAccount,
    pub bob: UserAccount,
    pub charlie: UserAccount,
    pub dude: UserAccount,
    pub eve: UserAccount,
    pub operator: UserAccount,
    pub farmer1: UserAccount,
    pub farmer2: UserAccount,
}

impl Users {
    pub fn init(e: &Env) -> Self {
        Self {
            alice: e.near.create_user(account_id("alice.near"), to_yocto("10000")),
            bob: e.near.create_user(account_id("bob.near"), to_yocto("10000")),
            charlie: e.near.create_user(account_id("charlie.near"), to_yocto("10000")),
            dude: e.near.create_user(account_id("dude.near"), to_yocto("10000")),
            eve: e.near.create_user(account_id("eve.near"), to_yocto("10000")),
            operator: e.near.create_user(account_id("operator.near"), to_yocto("10000")),
            farmer1: e.near.create_user(account_id("farmer1.near"), to_yocto("10000")),
            farmer2: e.near.create_user(account_id("farmer2.near"), to_yocto("10000")),
        }
    }
}
