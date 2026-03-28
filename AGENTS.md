# AGENTS.md — MISSION CONTROL

## 1. MISIÓN Y DOCTRINA (Rust/HFT)
Infraestructura crítica. Tolerancia Cero.
- **Hot-path:** Zero-alloc, zero `panic/unwrap`. Tokio + zero-copy.
- **Latencia:** <100µs.
- **Safety:** Air-gapped wallets. Prohibido leer disco en hot-path.

## 2. ESTRUCTURA CANÓNICA
- `core/` (Rust Engine) | `intelligence/` (ML/Signals) | `operational/` (DevOps) | `skills/` (HFT Tasks).

## 3. AGENT SKILLS (Auto-invoke)
| Action | Skill |
|--------|-------|
| Swap, Raydium, Solana, Jito, MEV | `raydium-v2`, `solana-jito-mev` |
| GitHub CI, PR Gate, Commits | `trading-ci`, `trading-pr-gate`, `trading-commit` |
| .rs Core, Execution Router | `rust-hft-patterns`, `skill-sync` |

## 4. SUBSISTEMAS
- `core/AGENTS.md` (Zero-Alloc) | `intelligence/AGENTS.md` (ML) | `operational/AGENTS.md` (Security).

*Antigravity System — Precision Execution*
