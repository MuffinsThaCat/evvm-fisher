# Security & Design Decisions (Fisher Contracts)

This document summarizes the key security and protocol design decisions for the Fisher contracts:

- `contracts/fisher/OptimizedFisher.sol`
- `contracts/fisher/HyperOptimizedFisher.sol`
- `contracts/fisher/LonsdaleiteOptimizedFisher.sol`

It is intended as an audit companion: it explains the intent behind key mechanisms, the threat models they address, and the tradeoffs.

---

## 1. Roles, Trust Model, and Threat Assumptions

### 1.1 Roles

- **Operator**
  - Posts Merkle roots.
  - Deposits yield tokens for each era.
  - Advances eras / performs operational actions.
  - Has emergency activation power (see E1).

- **Guardian** (recommended multisig)
  - Disputes malicious or incorrect Merkle roots.
  - Approves large sweeps.
  - Can trigger emergency mode via a delayed path.
  - Can remove themselves.
  - Can perform R2 dispute-count resets.

- **Users**
  - Claim yield using Merkle proofs (and commit-reveal where enforced).

### 1.2 Trust assumptions

- The system is designed to be safe under:
  - **Honest guardian** with potentially compromised/malicious operator.
  - **Honest operator** with potentially compromised/malicious guardian.

- If **both** operator and guardian are compromised/colluding, integrity of distributions cannot be guaranteed. The design focuses on:
  - Increasing transparency.
  - Adding delays and caps.
  - Adding break-glass tooling.
  - Preventing funds from being drained quickly.

---

## 2. Era Accounting Model (Strict Per-Era Pots)

### 2.1 Decision: strict per-era pots

We chose to enforce that each era has its own funding bucket.

- `eraTokenDeposit[era]` tracks the total tokens deposited to fund claims for that era.
- `eraTotalClaimed[era]` tracks how much of that era’s bucket has been consumed.

**Invariant (per era):**

- `eraTotalClaimed[era] <= eraTokenDeposit[era]`

Batch and single-claim flows enforce this by checking:

- `remainingForEra = eraTokenDeposit[era] - eraTotalClaimed[era]`
- revert if `remainingForEra < claimAmount`

This prevents **cross-era subsidization** where claims from era A could consume tokens intended for era B.

### 2.2 Definition of `eraTotalClaimed`: gross outflow

We define:

- `eraTotalClaimed[era]` = **gross token outflow** attempted/sent by the contract for that era.

This definition is robust under fee-on-transfer tokens (see §3): tokens may be taxed during transfer, but the contract’s balance decreases by the gross transfer amount.

---

## 3. Fee-on-Transfer Token Handling

### 3.1 Problem

Fee-on-transfer tokens can cause:

- Users receiving less than the Merkle amount.
- Accounting drift if state is updated as if users received the full amount.
- Underflow or inconsistent invariants if adjustments are performed incorrectly.

### 3.2 Decision: track “loss” separately

We treat the transfer tax as “precision loss” and track it separately:

- `eraPrecisionLoss[era]` accumulates fee-on-transfer loss.

The contracts measure actual received amounts using `balanceBefore` / `balanceAfter` on the user.

### 3.3 Slippage bounds

To avoid pathological tokens (or unexpected tax spikes), claims include slippage bounds:

- If `(gross - actualReceived) / gross` exceeds `MAX_SLIPPAGE_BPS`, revert.

This ensures users and the protocol are not silently operating under extreme token transfer taxes.

---

## 4. Merkle Leaf Domain Separation (Replay Protection)

### 4.1 Decision

Merkle leaves include:

- `block.chainid`
- `address(this)`
- `msg.sender`
- `era`
- `amount`
- `VERSION`

This prevents replay across:

- chains/forks,
- contracts,
- users,
- protocol versions.

---

## 5. Dispute System

### 5.1 Guardian disputes

Guardian can dispute a posted Merkle root within the dispute window.

Constraints:

- Dispute must occur within the configured time window.
- Rate-limited via a guardian dispute cooldown.

### 5.2 Replacement

Operator can replace disputed roots after a delay.

This prevents immediate “dispute → replace” in the same block and provides a reaction window.

---

## 6. Operator Dispute Revocation + R2 Recovery

### 6.1 Problem

A permanent operator lockout creates a liveness failure mode: the protocol can become stuck.

### 6.2 Decision: R2 reset with delay (guardian-gated)

We implemented a guardian-gated reset flow in all three contracts:

- `requestOperatorDisputeCountReset()`
- wait `MIN_DISPUTE_RESOLUTION_DELAY`
- `executeOperatorDisputeCountReset()`

This allows recovery from a revoked operator state while ensuring the reset cannot be executed instantly.

Events are emitted:

- `OperatorDisputeResetRequested`
- `OperatorDisputeCountReset`

---

## 7. Emergency Mode (E1)

### 7.1 Problem

Emergency mode is a critical circuit breaker, but it can also be abused as a DoS mechanism.

### 7.2 Decision: E1 (freeze fast, unfreeze cautiously)

We implemented E1 in all three contracts:

- Operator can **activate** emergency mode instantly.
- **Deactivation** is controlled:
  - If `guardian != address(0)`, only guardian may deactivate.
  - If guardian is not yet set, operator may deactivate (prevents permanent lock before guardian setup).

This ensures:

- fast response to real exploits,
- prevents operator from griefing by toggling emergency mode to block claims indefinitely.

---

## 8. Reentrancy and CEI

- Claim flows use `nonReentrant`.
- State changes are performed before external token transfers where feasible.

---

## 9. Deposit-After-Root Prevention

To prevent accounting manipulation, deposits are disallowed once a Merkle root is posted for an era:

- `require(eraMerkleRoot[era] == bytes32(0), "Root already posted")`

This prevents the operator from changing the funding basis after users can compute claims.

---

## 10. Large Sweeps (Guardian Approval)

Sweeping unclaimed funds is constrained:

- sweep delay enforced,
- large sweeps require guardian approval.

This reduces the blast radius of an operator key compromise.

---

## 11. Operational Notes

### 11.1 Guardian

Guardian should be a multisig, and its operational security is a primary real-world security dependency.

### 11.2 Monitoring

Monitor:

- emergency mode toggles,
- dispute events,
- root postings,
- dispute-count reset requests/executions,
- large sweep approvals.

---

## 12. Summary of Key Decisions

- Strict per-era pots enforced via `eraTokenDeposit` / `eraTotalClaimed`.
- `eraTotalClaimed` tracks gross token outflow; fee-on-transfer “loss” is tracked separately.
- Merkle leaves include chainid, contract, version (domain separation).
- Dispute lifecycle includes cooldowns and resolution delays.
- R2 allows guardian-gated dispute-count resets with delay.
- E1 emergency: operator can freeze instantly; guardian controls unfreeze once set.
