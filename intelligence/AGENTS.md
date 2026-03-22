# AGENTS.md — intelligence/ (The Lab)
> Overrides root AGENTS.md for all work inside intelligence/.

## Component Purpose
ML-driven signal generation. Research happens here; production signals
flow to core/ via ZMQ + Protobuf.

## Auto-invoke Skills
| Action                     | Skill       |
|----------------------------|-------------|
| Train or evaluate a model  | `ml-signal` |
| Add or modify datasets     | `ml-signal` |

## Validation Gate
No model ships to production without a validation report in
`intelligence/ml/scripts/operational/audits/`.
Minimum: Sharpe ≥ 1.5, MaxDD ≤ 15%, 90-day backtest.

## Data Integration & Protobuf Determinism
- Any change to the Python-to-Rust bridge (ZeroMQ + Protobuf) MUST prove it does not introduce serialization latency over 0.5ms.
