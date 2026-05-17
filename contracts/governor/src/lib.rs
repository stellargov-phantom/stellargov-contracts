#![no_std]
//! StellarGovernor — on-chain governance contract for Soroban.
//!
//! Inspired by Compound Governor Bravo, adapted for the Stellar/Soroban model.
//! Supports: proposal creation, voting, queuing into Timelock, and execution.
//!
//! Governance flow:
//!   1. Token holder calls propose() with a description and calldata
//!   2. Voting period begins (configurable ledger window)
//!   3. If quorum reached and majority For: proposal succeeds
//!   4. Proposer calls queue() → proposal enters Timelock
//!   5. After timelock delay: anyone calls execute()
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Env, String,
};

#[contracttype]
#[derive(Clone, PartialEq, Debug)]
pub enum ProposalState {
    Pending,    // created, voting not yet started
    Active,     // voting in progress
    Defeated,   // voting ended, quorum not reached or majority Against
    Succeeded,  // voting ended, quorum reached, majority For
    Queued,     // in Timelock
    Executed,   // executed
    Cancelled,  // cancelled by proposer
}

#[contracttype]
#[derive(Clone)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    pub description: String,
    pub for_votes: i128,
    pub against_votes: i128,
    pub abstain_votes: i128,
    pub start_ledger: u32,
    pub end_ledger: u32,
    pub state: ProposalState,
    pub eta_ledger: u32,  // timelock ETA (set when queued)
}

#[contracttype]
pub enum Key {
    Proposal(u64),
    Counter,
    VotingToken,
    TimelockContract,
    VotingPeriodLedgers,
    VotingDelayLedgers,
    ProposalThreshold,
    QuorumNumerator,
    HasVoted(u64, Address),   // (proposal_id, voter) → bool
}

#[contract]
pub struct StellarGovernor;

#[contractimpl]
impl StellarGovernor {
    /// Initialize the governor.
    ///
    /// * `voting_token`         — address of the governance token contract
    /// * `timelock_contract`    — address of the Timelock contract
    /// * `voting_delay_ledgers` — ledgers to wait before voting starts (~1 day = 17280)
    /// * `voting_period_ledgers`— ledgers for the voting window (~7 days = 120960)
    /// * `proposal_threshold`   — minimum tokens to create a proposal (in stroops)
    /// * `quorum_numerator`     — quorum as % of total supply (e.g. 4 = 4%)
    pub fn initialize(
        env: Env,
        voting_token: Address,
        timelock_contract: Address,
        voting_delay_ledgers: u32,
        voting_period_ledgers: u32,
        proposal_threshold: i128,
        quorum_numerator: u32,
    ) {
        assert!(!env.storage().instance().has(&Key::VotingToken),
            "already initialized");
        assert!(quorum_numerator <= 100, "quorum must be <= 100%");

        env.storage().instance().set(&Key::VotingToken, &voting_token);
        env.storage().instance().set(&Key::TimelockContract, &timelock_contract);
        env.storage().instance().set(&Key::VotingDelayLedgers, &voting_delay_ledgers);
        env.storage().instance().set(&Key::VotingPeriodLedgers, &voting_period_ledgers);
        env.storage().instance().set(&Key::ProposalThreshold, &proposal_threshold);
        env.storage().instance().set(&Key::QuorumNumerator, &quorum_numerator);
        env.storage().instance().set(&Key::Counter, &0u64);
    }

    /// Create a governance proposal.
    pub fn propose(
        env: Env,
        proposer: Address,
        description: String,
    ) -> u64 {
        proposer.require_auth();

        let delay: u32 = env.storage().instance()
            .get(&Key::VotingDelayLedgers).unwrap();
        let period: u32 = env.storage().instance()
            .get(&Key::VotingPeriodLedgers).unwrap();

        let id: u64 = env.storage().instance()
            .get(&Key::Counter).unwrap_or(0u64) + 1;
        env.storage().instance().set(&Key::Counter, &id);

        let current = env.ledger().sequence();
        let proposal = Proposal {
            id,
            proposer: proposer.clone(),
            description: description.clone(),
            for_votes: 0,
            against_votes: 0,
            abstain_votes: 0,
            start_ledger: current + delay,
            end_ledger: current + delay + period,
            state: ProposalState::Pending,
            eta_ledger: 0,
        };

        env.storage().persistent().set(&Key::Proposal(id), &proposal);
        env.events().publish(
            (symbol_short!("proposed"), id, proposer),
            description,
        );
        id
    }

    /// Cast a vote on a proposal.
    /// support: 0 = Against, 1 = For, 2 = Abstain
    pub fn cast_vote(
        env: Env,
        voter: Address,
        proposal_id: u64,
        support: u32,
        weight: i128,   // WAVE CONTRIBUTION GAP — Real implementation: fetch voter's token balance at snapshot from token contract
    ) {
        voter.require_auth();
        assert!(support <= 2, "invalid vote: 0=Against 1=For 2=Abstain");
        assert!(weight > 0, "no voting power");

        let voted_key = Key::HasVoted(proposal_id, voter.clone());
        assert!(!env.storage().temporary().has(&voted_key), "already voted");

        let mut proposal: Proposal = env.storage().persistent()
            .get(&Key::Proposal(proposal_id)).expect("proposal not found");

        let current = env.ledger().sequence();
        assert!(current >= proposal.start_ledger, "voting not started");
        assert!(current <= proposal.end_ledger, "voting ended");

        match support {
            1 => proposal.for_votes = proposal.for_votes.checked_add(weight).expect("overflow"),
            0 => proposal.against_votes = proposal.against_votes.checked_add(weight).expect("overflow"),
            2 => proposal.abstain_votes = proposal.abstain_votes.checked_add(weight).expect("overflow"),
            _ => unreachable!(),
        }

        env.storage().temporary().set(&voted_key, &true);
        env.storage().persistent().set(&Key::Proposal(proposal_id), &proposal);

        env.events().publish(
            (symbol_short!("voted"), proposal_id, voter),
            (support, weight),
        );
    }

    /// Finalise a proposal after voting ends.
    /// Updates state to Succeeded or Defeated.
    pub fn finalize(env: Env, proposal_id: u64) {
        let mut proposal: Proposal = env.storage().persistent()
            .get(&Key::Proposal(proposal_id)).expect("not found");

        assert!(env.ledger().sequence() > proposal.end_ledger,
            "voting still active");
        assert!(proposal.state == ProposalState::Active ||
                proposal.state == ProposalState::Pending,
            "already finalized");

        let quorum_num: u32 = env.storage().instance()
            .get(&Key::QuorumNumerator).unwrap();

        // WAVE CONTRIBUTION GAP — Real implementation: fetch total supply from token contract for accurate quorum check
        let total_votes = proposal.for_votes + proposal.against_votes + proposal.abstain_votes;
        let quorum_met = total_votes > 0 &&
            proposal.for_votes * 100 >= total_votes * quorum_num as i128;

        proposal.state = if quorum_met && proposal.for_votes > proposal.against_votes {
            ProposalState::Succeeded
        } else {
            ProposalState::Defeated
        };

        env.storage().persistent().set(&Key::Proposal(proposal_id), &proposal);
        env.events().publish(
            (symbol_short!("final"), proposal_id),
            proposal.state.clone(),
        );
    }

    /// Queue a succeeded proposal into the Timelock.
    pub fn queue(env: Env, proposal_id: u64, timelock_delay: u32) {
        let mut proposal: Proposal = env.storage().persistent()
            .get(&Key::Proposal(proposal_id)).expect("not found");

        assert!(proposal.state == ProposalState::Succeeded, "not succeeded");

        let eta = env.ledger().sequence() + timelock_delay;
        proposal.state = ProposalState::Queued;
        proposal.eta_ledger = eta;
        env.storage().persistent().set(&Key::Proposal(proposal_id), &proposal);
        env.events().publish(
            (symbol_short!("queued"), proposal_id),
            eta,
        );
    }

    /// Execute a queued proposal after its timelock ETA.
    pub fn execute(env: Env, proposal_id: u64) {
        let mut proposal: Proposal = env.storage().persistent()
            .get(&Key::Proposal(proposal_id)).expect("not found");

        assert!(proposal.state == ProposalState::Queued, "not queued");
        assert!(env.ledger().sequence() >= proposal.eta_ledger, "timelock not expired");

        proposal.state = ProposalState::Executed;
        env.storage().persistent().set(&Key::Proposal(proposal_id), &proposal);
        env.events().publish(
            (symbol_short!("executed"), proposal_id),
            (),
        );
        // WAVE CONTRIBUTION GAP — Real implementation: invoke Timelock.execute() which calls the proposal's target contracts
    }

    /// Cancel a proposal. Only proposer can cancel before execution.
    pub fn cancel(env: Env, proposal_id: u64) {
        let mut proposal: Proposal = env.storage().persistent()
            .get(&Key::Proposal(proposal_id)).expect("not found");

        proposal.proposer.require_auth();
        assert!(proposal.state != ProposalState::Executed, "already executed");

        proposal.state = ProposalState::Cancelled;
        env.storage().persistent().set(&Key::Proposal(proposal_id), &proposal);
        env.events().publish((symbol_short!("cancel"), proposal_id), ());
    }

    pub fn get_proposal(env: Env, id: u64) -> Proposal {
        env.storage().persistent().get(&Key::Proposal(id)).expect("not found")
    }

    pub fn proposal_count(env: Env) -> u64 {
        env.storage().instance().get(&Key::Counter).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env, String};

    fn setup() -> (Env, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let token = Address::generate(&env);
        let timelock = Address::generate(&env);
        (env, token, timelock)
    }

    #[test]
    fn propose_and_vote() {
        let (env, token, timelock) = setup();
        let proposer = Address::generate(&env);
        let voter = Address::generate(&env);
        let cid = env.register_contract(None, StellarGovernor);
        let client = StellarGovernorClient::new(&env, &cid);

        client.initialize(&token, &timelock, &0u32, &100u32, &1000i128, &4u32);
        let desc = String::from_str(&env, "Increase protocol fee from 0.1% to 0.2%");
        let pid = client.propose(&proposer, &desc);

        client.cast_vote(&voter, &pid, &1u32, &5000i128); // For
        assert_eq!(client.get_proposal(&pid).for_votes, 5000);
    }

    #[test]
    fn cancelled_proposal() {
        let (env, token, timelock) = setup();
        let proposer = Address::generate(&env);
        let cid = env.register_contract(None, StellarGovernor);
        let client = StellarGovernorClient::new(&env, &cid);
        client.initialize(&token, &timelock, &0u32, &100u32, &1000i128, &4u32);
        let desc = String::from_str(&env, "Test cancel");
        let pid = client.propose(&proposer, &desc);
        client.cancel(&pid);
        assert_eq!(client.get_proposal(&pid).state, ProposalState::Cancelled);
    }
}
