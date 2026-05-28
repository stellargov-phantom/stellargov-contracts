# StellarGov — Drips Wave 5 Task Backlog

Welcome to the StellarGov Drips Wave task backlog! Contribute to on-chain DAOs and governance on Soroban.

---

## 🟢 Trivial Tasks (100 Points)

### SGV-T01: Description Word Counter for Proposal Creation
* **Description:** Implement a simple text validator and visual counter in the proposal creation interface to prevent users from exceeding limits.
* **Complexity:** Trivial (100 pts)
* **Status:** Open 🚀
* **Files to Edit:**
  * `stellargov-app/src/pages/ProposalsPage.tsx`

---

## 🟡 Medium Tasks (150 Points)

### SGV-M01: Delegation History Telemetry Page
* **Description:** Implement a dashboard view displaying historic delegation distributions and identifying dominant voting weights.
* **Complexity:** Medium (150 pts)
* **Status:** Open 🚀
* **Files to Edit:**
  * `stellargov-app/src/pages/DelegatePage.tsx`
  * `stellargov-app/src/components/DelegationMap.tsx`
* **Acceptance Criteria:**
  * Displays a historical timeline of voting power delegation with interactive cards.

---

## 🔴 High Tasks (200 Points)

### SGV-H01: Timelock Operations Integration
* **Description:** Integrate the `StellarTimelock` contract execution trigger into the `StellarGovernor` contract, forcing proposals to complete timelock delay verification before dispatching assets from the treasury.
* **Complexity:** High (200 pts)
* **Status:** Open 🚀
* **Files to Edit:**
  * `stellargov-contracts/contracts/governor/src/lib.rs`
  * `stellargov-contracts/contracts/governor/src/types.rs`
* **Acceptance Criteria:**
  * `execute()` function correctly verifies queue state against the timelock contract address before proceeding.
