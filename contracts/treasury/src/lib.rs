#![no_std]
//! StellarTreasury — multi-asset DAO treasury controlled by the StellarGovernor.
//!
//! All spending requires governance approval via the Timelock.
//! Tracks balances across multiple SAC tokens, emits events for
//! on-chain accounting, and supports configurable spending limits
//! for fast-track operational expenses (streaming payments, payroll).
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Env, String,
};

#[contracttype]
#[derive(Clone)]
pub struct Allocation {
    pub recipient: Address,
    pub asset: Address,
    pub amount: i128,
    pub description: String,
    pub executed: bool,
}

#[contracttype]
pub enum Key {
    Governor,
    Timelock,
    Allocation(u64),
    AllocationCounter,
    /// Per-asset balance tracking
    AssetBalance(Address),
    /// Streaming payment allowance per address
    StreamAllowance(Address),
}

#[contract]
pub struct StellarTreasury;

#[contractimpl]
impl StellarTreasury {
    pub fn initialize(env: Env, governor: Address, timelock: Address) {
        governor.require_auth();
        assert!(!env.storage().instance().has(&Key::Governor), "already initialized");
        env.storage().instance().set(&Key::Governor, &governor);
        env.storage().instance().set(&Key::Timelock, &timelock);
        env.storage().instance().set(&Key::AllocationCounter, &0u64);
    }

    /// Record an inbound deposit (called when tokens are sent to treasury).
    pub fn record_deposit(env: Env, asset: Address, amount: i128, depositor: Address) {
        assert!(amount > 0, "amount must be positive");
        let bal: i128 = env.storage().persistent()
            .get(&Key::AssetBalance(asset.clone())).unwrap_or(0);
        env.storage().persistent()
            .set(&Key::AssetBalance(asset.clone()), &(bal.checked_add(amount).expect("overflow")));
        env.events().publish(
            (symbol_short!("deposit"), asset, depositor),
            amount,
        );
    }

    /// Create a governance-approved allocation for spending.
    /// Only the Timelock (acting on Governor approval) can create allocations.
    pub fn create_allocation(
        env: Env,
        recipient: Address,
        asset: Address,
        amount: i128,
        description: String,
    ) -> u64 {
        let timelock: Address = env.storage().instance()
            .get(&Key::Timelock).expect("not initialized");
        timelock.require_auth();

        assert!(amount > 0, "amount must be positive");

        let bal: i128 = env.storage().persistent()
            .get(&Key::AssetBalance(asset.clone())).unwrap_or(0);
        assert!(bal >= amount, "insufficient treasury balance");

        let id: u64 = env.storage().instance()
            .get(&Key::AllocationCounter).unwrap_or(0u64) + 1;
        env.storage().instance().set(&Key::AllocationCounter, &id);

        let allocation = Allocation {
            recipient: recipient.clone(),
            asset: asset.clone(),
            amount,
            description: description.clone(),
            executed: false,
        };

        env.storage().persistent().set(&Key::Allocation(id), &allocation);
        env.events().publish(
            (symbol_short!("alloc"), id, recipient),
            (asset, amount, description),
        );
        id
    }

    /// Execute an approved allocation — transfer tokens to recipient.
    pub fn execute_allocation(env: Env, allocation_id: u64) {
        let mut allocation: Allocation = env.storage().persistent()
            .get(&Key::Allocation(allocation_id)).expect("not found");

        assert!(!allocation.executed, "already executed");

        // Deduct from treasury balance
        let bal: i128 = env.storage().persistent()
            .get(&Key::AssetBalance(allocation.asset.clone())).unwrap_or(0);
        assert!(bal >= allocation.amount, "insufficient balance");

        env.storage().persistent().set(
            &Key::AssetBalance(allocation.asset.clone()),
            &(bal - allocation.amount),
        );

        allocation.executed = true;
        env.storage().persistent().set(&Key::Allocation(allocation_id), &allocation);

        // WAVE CONTRIBUTION GAP — Real implementation: call SAC token contract to transfer allocation.amount
        // to allocation.recipient
        env.events().publish(
            (symbol_short!("exec_al"), allocation_id, allocation.recipient.clone()),
            allocation.amount,
        );
    }

    /// Set a streaming payment allowance for operational expenses.
    /// Requires Timelock auth — set via governance for recurring costs (payroll, etc.).
    pub fn set_stream_allowance(
        env: Env,
        recipient: Address,
        monthly_amount: i128,
    ) {
        let timelock: Address = env.storage().instance()
            .get(&Key::Timelock).expect("not initialized");
        timelock.require_auth();
        env.storage().persistent()
            .set(&Key::StreamAllowance(recipient.clone()), &monthly_amount);
        env.events().publish(
            (symbol_short!("stream"), recipient),
            monthly_amount,
        );
    }

    pub fn get_balance(env: Env, asset: Address) -> i128 {
        env.storage().persistent()
            .get(&Key::AssetBalance(asset)).unwrap_or(0)
    }

    pub fn get_allocation(env: Env, id: u64) -> Allocation {
        env.storage().persistent().get(&Key::Allocation(id)).expect("not found")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env, String};

    #[test]
    fn deposit_and_balance() {
        let env = Env::default();
        env.mock_all_auths();
        let governor  = Address::generate(&env);
        let timelock  = Address::generate(&env);
        let depositor = Address::generate(&env);
        let asset     = Address::generate(&env);

        let cid = env.register_contract(None, StellarTreasury);
        let client = StellarTreasuryClient::new(&env, &cid);
        client.initialize(&governor, &timelock);
        client.record_deposit(&asset, &10_000i128, &depositor);
        assert_eq!(client.get_balance(&asset), 10_000);
    }

    #[test]
    fn create_and_execute_allocation() {
        let env = Env::default();
        env.mock_all_auths();
        let governor   = Address::generate(&env);
        let timelock   = Address::generate(&env);
        let depositor  = Address::generate(&env);
        let recipient  = Address::generate(&env);
        let asset      = Address::generate(&env);

        let cid = env.register_contract(None, StellarTreasury);
        let client = StellarTreasuryClient::new(&env, &cid);
        client.initialize(&governor, &timelock);
        client.record_deposit(&asset, &5_000i128, &depositor);

        let desc = String::from_str(&env, "Q2 contractor payment");
        let alloc_id = client.create_allocation(&recipient, &asset, &2_000i128, &desc);
        client.execute_allocation(&alloc_id);

        assert_eq!(client.get_balance(&asset), 3_000);
        assert!(client.get_allocation(&alloc_id).executed);
    }
}
