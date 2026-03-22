# AGENTS.md — core/ (The Chassis)
> Overrides root AGENTS.md for all work inside core/.

## Component Purpose
Low-latency Rust execution engine for Solana trading.
Every microsecond counts. Precision over convenience.

## Module Map
```
core/src/
├── bin/        → Entry points: one per execution concern
├── engine/     → Order routing, risk management, position tracking
├── sensors/    → Geyser slot subscription, RPC telemetry
├── telegram/   → Operator notifications
└── generated/  → Protobuf output (DO NOT EDIT)
```

## Auto-invoke Skills
| Action                        | Skill            |
|-------------------------------|------------------|
| Edit any `.rs` file           | `rust-hft-patterns`    |
| Work with Geyser / RPC / DEX  | `solana-jito-mev` |

## Build Commands
```bash
cargo build --release
cargo clippy -- -D warnings
cargo test
cargo audit
```

## Protobuf Regeneration
```bash
cd core && cargo build  # triggers build.rs auto-generation
```
Never edit files in `src/generated/` manually.
