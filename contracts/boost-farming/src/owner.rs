use crate::*;

#[derive(Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(feature = "test", derive(Clone))]
pub struct ImportSeedInfo {
    pub seed_id: String,
    pub seed_decimal: u32,
    pub amount: U128,
    pub min_deposit: U128,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(feature = "test", derive(Clone))]
pub struct ImportFarmerInfo {
    pub farmer_id: AccountId,
    pub rewards: HashMap<AccountId, U128>,
    pub seeds: HashMap<SeedId, U128>,
}

impl Contract {
    pub fn assert_owner(&self) {
        require!(
            env::predecessor_account_id() == self.data().owner_id,
            E002_NOT_ALLOWED
        );
    }
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn set_owner(&mut self, owner_id: AccountId) {
        assert_one_yocto();
        self.assert_owner();
        self.data_mut().owner_id = owner_id;
    }

    #[payable]
    pub fn pause_contract(&mut self) {
        assert_one_yocto();
        self.assert_owner();

        if self.data().state == RunningState::Running {
            log!("Contract paused by {}", env::predecessor_account_id());       
            self.data_mut().state = RunningState::Paused;
        } else {
            log!("Contract state is already in Paused");
        }
    }

    #[payable]
    pub fn resume_contract(&mut self) {
        assert_one_yocto();
        self.assert_owner();

        if self.data().state == RunningState::Paused {
            log!("Contract resumed by {}", env::predecessor_account_id());       
            self.data_mut().state = RunningState::Running;
        } else {
            log!("Contract state is already in Running");
        }
    }

    /// Extend operators. Only can be called by owner.
    #[payable]
    pub fn extend_operators(&mut self, operators: Vec<AccountId>) {
        assert_one_yocto();
        self.assert_owner();
        for operator in operators {
            self.data_mut().operators.insert(&operator);
        }
    }

    /// Remove operators. Only can be called by owner.
    #[payable]
    pub fn remove_operators(&mut self, operators: Vec<AccountId>) {
        assert_one_yocto();
        self.assert_owner();
        for operator in operators {
            self.data_mut().operators.remove(&operator);
        }
    }

    /// Should only be called by this contract on migration.
    /// This is NOOP implementation. KEEP IT if you haven't changed contract state.
    /// If you have, you need to implement migration from old state 
    /// (keep the old struct with different name to deserialize it first).
    /// After migration goes live, revert back to this implementation for next updates.
    #[init(ignore_state)]
    #[private]
    pub fn migrate() -> Self {
        let mut contract: Contract = env::state_read().expect(E003_NOT_INIT);
        // see if ContractData need upgrade
        contract.data = 
        match contract.data {
            VersionedContractData::V0100(data) => VersionedContractData::V0101(data.into()),
            VersionedContractData::V0101(data) => VersionedContractData::V0101(data),
        };
        contract
    }
}

#[cfg(target_arch = "wasm32")]
mod upgrade {
    use near_sdk::Gas;
    use near_sys as sys;

    use super::*;

    /// Gas for calling migration call.
    pub const GAS_FOR_MIGRATE_CALL: Gas = Gas(5_000_000_000_000);

    /// Self upgrade and call migrate, optimizes gas by not loading into memory the code.
    /// Takes as input non serialized set of bytes of the code.
    #[no_mangle]
    pub fn upgrade() {
        env::setup_panic_hook();
        let contract: Contract = env::state_read().expect("ERR_CONTRACT_IS_NOT_INITIALIZED");
        contract.assert_owner();
        let current_id = env::current_account_id().as_bytes().to_vec();
        let method_name = "migrate".as_bytes().to_vec();
        unsafe {
            // Load input (wasm code) into register 0.
            sys::input(0);
            // Create batch action promise for the current contract ID
            let promise_id =
                sys::promise_batch_create(current_id.len() as _, current_id.as_ptr() as _);
            // 1st action in the Tx: "deploy contract" (code is taken from register 0)
            sys::promise_batch_action_deploy_contract(promise_id, u64::MAX as _, 0);
            // 2nd action in the Tx: call this_contract.migrate() with remaining gas
            let attached_gas = env::prepaid_gas() - env::used_gas() - GAS_FOR_MIGRATE_CALL;
            sys::promise_batch_action_function_call(
                promise_id,
                method_name.len() as _,
                method_name.as_ptr() as _,
                0 as _,
                0 as _,
                0 as _,
                attached_gas.0,
            );
        }
    }
}