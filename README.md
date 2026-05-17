# StellarGov Contracts: Core Governance & Treasury Logic

[![Stellar Wave 5](https://img.shields.io/badge/Drips_Wave-5-blueviolet?style=for-the-badge)](https://www.drips.network/wave)
[![Language: Rust](https://img.shields.io/badge/Language-Rust-dea584?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg?style=for-the-badge)](https://opensource.org/licenses/Apache-2.0)

**The on-chain core of the StellarGov governance suite. Secure, modular Soroban smart contracts implementing checkpoints, timelocks, and treasury custody.**

---

# 🗳️ Technical Overview

`stellargov-contracts` brings reliable, Compound-grade decentralized governance to the Stellar Network. Built natively on the **Soroban Smart Contract platform** using Rust, these contracts are designed to enforce transparent, on-chain parameters for proposals, voting, timelocks, and decentralized asset custody.

### Core Contracts:
1.  **`VotingToken`:** A token wrapper that records voting power checkpoints. Every transfer updates a checkpoint list, letting the Governor query historical voting balances at a specific past ledger block, completely preventing double-voting attacks.
2.  **`Governor`:** Enforces the proposal lifecycle (submission, voting delay, voting, queueing, execution). Manages parameters such as voting delay, voting period, proposal threshold, and quorum.
3.  **`Timelock`:** A decentralized queue contract that holds passed proposals for a configured execution delay (e.g., 48 hours). This acts as a vital security buffer, allowing community members to withdraw or react before changes are executed on-chain.
4.  **`Treasury`:** The decentralized vault that stores DAO funds. It is cryptographically configured to only execute state mutations and payouts initiated by the verified `Timelock` contract.

---

# 🏗️ Internal Architecture & State Transitions

```mermaid
graph TD
    subgraph "Token Mechanics"
        VT[VotingToken Contract] -- "Records Checkpoint Balances" --> VT
    end

    subgraph "On-Chain Decision Lifecycle"
        VT -- "1. Validate Voting Power" --> Gov[Governor Contract]
        Gov -- "2. Check Quorum & Passes" --> Gov
        Gov -- "3. Queue Transaction" --> TL[Timelock Contract]
        TL -- "4. Delay Period (48h)" --> TL
        TL -- "5. Execute Call" --> TR[Treasury Contract]
    end

    subgraph "External Interactions"
        TR -- "Releases Funds / Calls Contracts" --> Ext[Target Contracts / Recipient]
    end
```

---

# 📋 Detailed Smart Contract Specification

### 1. Governor State Machine
Proposals in the `Governor` contract transition through the following states defined by the `ProposalState` enum:
*   `Pending`: Proposal created, waiting for `voting_delay` ledgers to pass.
*   `Active`: Voting is open. Members can cast `For`, `Against`, or `Abstain` votes.
*   `Defeated`: Voting period ended, and the proposal failed to meet the quorum or support threshold.
*   `Succeeded`: Proposal met both quorum and support thresholds. Ready to be queued.
*   `Queued`: Proposal successfully queued in the Timelock.
*   `Executed`: Proposal actions successfully executed.
*   `Expired`: Proposal passed but was not executed before the grace period ended.

### 2. Checkpoint Balance Storage Structure
To prevent voting balance manipulation within the same ledger (e.g., flash loans or rapid token recycling), the `VotingToken` contract leverages chronological checkpoint arrays stored in the contract's persistent storage:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Checkpoint {
    pub ledger: u32,
    pub votes: i128,
}
```

When a user transfers tokens, the contract updates their checkpoints:
1.  **Sender Checkpoint:** A new `Checkpoint` is appended with the current ledger index and the decremented voting balance.
2.  **Recipient Checkpoint:** A new `Checkpoint` is appended with the current ledger index and the incremented voting balance.
3.  **Binary Search Lookup:** During proposal queries, the Governor invokes `get_past_votes(user, proposal_start_ledger)` which performs a fast binary search over the checkpoints array to determine the user's exact balance at that past block height.

---

# 📋 Governance Parameter Configuration

The `Governor` contract exposes highly configurable parameters to fit institutional and community needs:

| Parameter | Type | Default Value | Description |
| :--- | :--- | :--- | :--- |
| **`voting_delay`** | `u32` (Ledgers) | `17,280` (~24 hours) | Number of ledgers between proposal submission and voting opening. |
| **`voting_period`** | `u32` (Ledgers) | `51,840` (~3 days) | Length of time (in ledgers) that voting remains open. |
| **`proposal_threshold`**| `i128` (Tokens)  | `10,000 VT` | Minimum voting power required to submit a proposal. |
| **`quorum_votes`** | `i128` (Tokens)  | `100,000 VT` (10%) | Minimum number of "For" votes required for a proposal to pass. |
| **`timelock_delay`** | `u64` (Seconds)  | `172,800` (48 hours)| Required execution delay enforced by the Timelock contract. |

---

# 📂 Repository Structure

```text
stellargov-contracts/
├── contracts/
│   ├── governor/         # Proposal lifecycle, quorum, and voting checks
│   │   └── src/
│   │       ├── lib.rs    # Core state machine functions
│   │       └── types.rs  # Shared parameter enums and structures
│   ├── timelock/         # Queue execution delays and admin functions
│   ├── treasury/         # Vault asset storage and execution controls
│   └── voting_token/     # Checkpoint-based token ledger logic
├── Cargo.toml            # Workspace manifest and dependencies
└── README.md             # You are here
```

---

# 🛠️ Development, Compilation & Testing

### 1. Prerequisites
Ensure you have the following installed locally:
*   Rust (v1.75+) with `wasm32-unknown-unknown` target.
*   Stellar CLI (v21.0.0+) for local contract deployments.

### 2. Local Setup & Building
Clone the repository and compile the workspace WASM files:
```bash
git clone https://github.com/stellargov-phantom/stellargov-contracts.git
cd stellargov-contracts
cargo build --target wasm32-unknown-unknown --release
```
Optimized `.wasm` binaries will be output to `target/wasm32-unknown-unknown/release/`.

### 3. Running Unit Tests
The contracts suite contains comprehensive unit tests verifying proposal lifecycles, vote thresholds, checkpoints, and timelocks:
```bash
cargo test
```

---

# 📄 License

This project is licensed under the **Apache License 2.0**.
