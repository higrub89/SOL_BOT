# V1 Operating Contract

Version: 1.0
Date: 2026-03-15
Status: Approved for implementation

## 1. Objective

This document defines the non-negotiable operating contract for The Chassis V1.
It is the source of truth for:
- Safety and capital exposure boundaries.
- Latency and reliability SLOs.
- Runtime degradation behavior.
- Dependency boundaries between crates.
- Production readiness gates.

Any implementation that conflicts with this contract is considered out-of-spec.

## 2. Scope

In scope for V1:
- Solana execution path with deterministic replay.
- Jito submission path with bounded tip policy.
- Multi-provider market data ingestion with failover.
- Automated risk classification for preflight policy switching.
- Shadow, paper, canary, and production operating modes.

Out of scope for V1:
- Multi-chain execution.
- Manual discretionary overrides in hot path.
- Unbounded adaptive behavior without risk constraints.

## 3. SLOs and Budgets

### 3.1 Latency SLOs

Latency is measured as four stages:
- T0: ingest -> decision
- T1: decision -> tx build + sign
- T2: send -> provider ack
- T3: ack -> observed inclusion delta

SLO policy:
- T0 and T1 are mandatory internal gates for release.
- T2 and T3 are environment-sensitive and must be tracked per topology profile.
- p99 and p99.9 jitter must be reported for every stage.

### 3.2 Reliability SLOs

- RTO (crash to operational): <= 90 seconds.
- RPO (state loss in replay domain): <= 1 second of ingest events.
- False-positive emergency halt rate: < 1 per 7 days.

### 3.3 Capital Exposure SLOs

- Maximum hot-wallet exposure is capped by policy and never equals treasury balance.
- Maximum per-trade loss, per-epoch loss, and daily loss are hard limits in `core-domain`.
- Breaching any hard limit forces `SAFE` or `HALT` transition.

## 4. Operating Modes and Degradation Model

The runtime mode is a first-class domain primitive:
- NORMAL: full strategy set, target latency profile.
- CONSTRAINED: reduced strategy set, reduced position size, stricter risk checks.
- SAFE: no new positions, only risk-reducing actions (hedge/close/cancel).
- HALT: trading disabled, control plane only.

Automatic transitions are driven by health signals:
- Clock drift > 50 ms.
- Slot lag threshold breached.
- Provider quorum degraded.
- Reject burst or anomalous slippage.
- Internal latency SLO breach sustained over rolling window.

Transition requirements:
- Every transition is event-logged with reason code.
- Recovery transitions require explicit health recovery criteria.

## 5. Key Management Policy (First-Class)

### 5.1 Wallet Roles

- Cold wallet: treasury custody only, never used in hot path.
- Hot wallet: bounded operational capital for active execution.
- Emergency wallet policy: pre-defined destination for emergency capital evacuation.

### 5.2 Authorization and Rotation

- Transfers from cold to hot require explicit operator authorization and audit logging.
- Hot key rotation is mandatory on anomaly triggers (compromise suspicion, signing anomalies, unauthorized access indicators).
- Rotation procedure must be executable within RTO constraints.

### 5.3 Exposure Limits

- Hot wallet max balance is a hard config limit with startup validation.
- Any attempt to exceed exposure limit is blocked and escalated.

## 6. Network Topology as Design Input

Topology is selected before implementation and versioned as deployment profile.

Mandatory profile fields:
- Region and provider.
- Estimated RTT to Jito endpoints.
- Estimated RTT to selected RPC providers.
- CPU model, core pinning plan, kernel/network tuning profile.

Policy:
- T2/T3 SLO targets are profile-specific.
- Production promotion requires passing canary under the exact target profile.

## 7. Deterministic Execution and Replay

### 7.1 Canonical Event Log

- Event log is append-only.
- Log write path is separated from decision path.
- Decision path cannot block on disk/network logging.
- Each decision carries deterministic `decision_hash` and input reference.

### 7.2 Crash Recovery Protocol

On restart:
1. Load latest validated snapshot.
2. Replay canonical event log from snapshot checkpoint.
3. Reconcile on-chain/open-order state.
4. Apply recovery policy for orphan or unknown orders.
5. Enter `SAFE` until reconciliation is complete, then promote by health checks.

Order reconciliation policy is explicit:
- Cancel unknown stale orders if cancel path is healthy.
- Inherit valid open risk-reducing orders.
- Never open new risk-increasing positions during reconciliation.

## 8. Tip/MEV Strategy Contract

Tip logic is part of execution contract, not ad-hoc config.

Inputs:
- Expected trade edge.
- Estimated competition intensity.
- Current latency profile and inclusion backlog signal.
- Risk budget and max tip policy.

Outputs:
- Computed tip amount.
- Submission urgency class.

Limits:
- `max_tip_percentage` is enforced in `core-domain`.
- Any computed tip above policy limit is clipped and logged.
- Repeated clipping is treated as strategy-health degradation signal.

## 9. Preflight Policy via RiskClassifier

Two policy levels:
- Policy A: mandatory preflight simulation.
- Policy B: adaptive/conditional preflight.

`RiskClassifier` escalation rules:
- Promote B -> A on reject burst, anomalous slippage, provider instability, or recent reconciliation events.
- Demote A -> B only after rolling-window stability criteria are met.

The classifier state is persisted and included in replay artifacts.

## 10. Data Provider Redundancy and Failover

Minimum requirement:
- At least two independent market data providers.

Behavior:
- If one provider fails, switch to degraded quorum and enter `CONSTRAINED`.
- If all primary feeds fail, transition to `HALT`.
- Failover decisions are deterministic and event-logged.

Provider policy includes:
- Priority order.
- Health scoring.
- Cooldown and rejoin criteria.

## 11. Crate Dependency Contract

Required boundaries:
- `strategy` and `execution` communicate through `core-domain` contracts only.
- `execution` must not depend on `strategy`.
- Risk rules live in `core-domain`/`risk`, not in adapters.

Enforcement:
- Workspace checks for dependency violations in CI.
- `deny(unused_crate_dependencies)` across crates.
- Architecture tests must fail build on forbidden edges.

## 12. Test Pyramid (Mandatory)

Required levels:
- Unit tests: per crate, deterministic logic.
- Integration tests: inter-crate contracts with mocked providers.
- Simulation tests: frozen datasets and deterministic replay assertions.
- Chaos tests: reconnect, feed loss, RPC degradation, partial provider failure.
- Benchmark regression tests: threshold-based fail in CI for hot-path latency regressions.

Coverage policy:
- Core/risk/execution coverage targets are quality-gated.
- Replay equivalence tests are mandatory for release.

## 13. Phase Gates (Go/No-Go)

- Gate A (spec): operating contract approved, topology selected, key policy approved.
- Gate B (baseline): T0/T1 baseline passes target on selected topology profile.
- Gate C (architecture): crate dependency contract enforced by CI.
- Gate D (execution): deterministic replay passes under load.
- Gate E (safety): degradation transitions verified by chaos scenarios.
- Gate F (go-live): shadow and canary pass for consecutive windows.

No gate can be bypassed by schedule pressure.

## 14. First Sprint Execution Split

### Day 1 (Decisions only)

- Finalize operating contract.
- Freeze topology profile.
- Freeze key management policy.
- Freeze crate dependency contract.

### Day 2 (Code only)

- Implement latency baseline harness.
- Implement CI skeleton and architecture checks.
- Add initial replay scaffolding and event schema.

No architecture decisions are made on Day 2.

## 15. Change Control

Any change to this contract requires:
- Written ADR with impact analysis.
- Updated tests and gate criteria if behavior changes.
- Explicit owner and rollback plan.
