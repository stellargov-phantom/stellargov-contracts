#![no_std]
//! StellarTimelock — enforces a mandatory delay between governance approval
//! and execution. Gives stakeholders time to exit before changes take effect.
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Env, String,
};

#[contracttype]
#[derive(Clone, PartialEq, Debug)]
pub enum OperationState { Pending, Ready, Done, Cancelled }

#[contracttype]
#[derive(Clone)]
pub struct Operation {
    pub id: u64,
    pub target: Address,
    pub function_name: String,
    pub eta_ledger: u32,
    pub state: OperationState,
    pub proposer: Address,
}

#[contracttype]
pub enum Key {
    Operation(u64),
    Counter,
    Governor,
    MinDelay,
}

#[contract]
pub struct StellarTimelock;

#[contractimpl]
impl StellarTimelock {
    pub fn initialize(env: Env, governor: Address, min_delay_ledgers: u32) {
        assert!(!env.storage().instance().has(&Key::Governor), "already initialized");
        assert!(min_delay_ledgers >= 100, "min delay too short");
        env.storage().instance().set(&Key::Governor, &governor);
        env.storage().instance().set(&Key::MinDelay, &min_delay_ledgers);
        env.storage().instance().set(&Key::Counter, &0u64);
    }

    /// Schedule an operation. Only the Governor can schedule.
    pub fn schedule(
        env: Env,
        target: Address,
        function_name: String,
        delay_ledgers: u32,
        proposer: Address,
    ) -> u64 {
        let governor: Address = env.storage().instance()
            .get(&Key::Governor).expect("not initialized");
        governor.require_auth();

        let min_delay: u32 = env.storage().instance()
            .get(&Key::MinDelay).unwrap();
        assert!(delay_ledgers >= min_delay, "delay too short");

        let id: u64 = env.storage().instance()
            .get(&Key::Counter).unwrap_or(0u64) + 1;
        env.storage().instance().set(&Key::Counter, &id);

        let eta = env.ledger().sequence() + delay_ledgers;
        let op = Operation {
            id,
            target: target.clone(),
            function_name: function_name.clone(),
            eta_ledger: eta,
            state: OperationState::Pending,
            proposer,
        };

        env.storage().persistent().set(&Key::Operation(id), &op);
        env.events().publish(
            (symbol_short!("sched"), id, target),
            (function_name, eta),
        );
        id
    }

    /// Execute a scheduled operation after its ETA.
    pub fn execute(env: Env, operation_id: u64) {
        let mut op: Operation = env.storage().persistent()
            .get(&Key::Operation(operation_id)).expect("not found");

        assert!(op.state == OperationState::Pending, "not pending");
        assert!(env.ledger().sequence() >= op.eta_ledger, "not ready");

        op.state = OperationState::Done;
        env.storage().persistent().set(&Key::Operation(operation_id), &op);

        // WAVE CONTRIBUTION GAP — Real implementation: invoke op.target contract with op.function_name
        env.events().publish(
            (symbol_short!("exec"), operation_id, op.target),
            op.function_name,
        );
    }

    /// Cancel a scheduled operation. Only governor can cancel.
    pub fn cancel(env: Env, operation_id: u64) {
        let governor: Address = env.storage().instance()
            .get(&Key::Governor).expect("not initialized");
        governor.require_auth();

        let mut op: Operation = env.storage().persistent()
            .get(&Key::Operation(operation_id)).expect("not found");
        assert!(op.state == OperationState::Pending, "not pending");
        op.state = OperationState::Cancelled;
        env.storage().persistent().set(&Key::Operation(operation_id), &op);
        env.events().publish((symbol_short!("cancel"), operation_id), ());
    }

    pub fn get_operation(env: Env, id: u64) -> Operation {
        env.storage().persistent().get(&Key::Operation(id)).expect("not found")
    }

    pub fn is_ready(env: Env, id: u64) -> bool {
        let op: Operation = env.storage().persistent()
            .get(&Key::Operation(id)).expect("not found");
        op.state == OperationState::Pending &&
        env.ledger().sequence() >= op.eta_ledger
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env, String};

    #[test]
    fn schedule_and_check_ready() {
        let env = Env::default();
        env.mock_all_auths();
        let governor = Address::generate(&env);
        let target = Address::generate(&env);
        let proposer = Address::generate(&env);
        let cid = env.register_contract(None, StellarTimelock);
        let client = StellarTimelockClient::new(&env, &cid);
        client.initialize(&governor, &100u32);
        let fname = String::from_str(&env, "set_fee");
        let id = client.schedule(&target, &fname, &100u32, &proposer);
        // Not ready yet (ETA is in the future for ledger sequence 0+100)
        // In a real test we'd advance the ledger
        let op = client.get_operation(&id);
        assert_eq!(op.state, OperationState::Pending);
    }
}
