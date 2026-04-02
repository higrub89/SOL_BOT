# AGENTS.md — MISSION CONTROL

> [!IMPORTANT]
> **MCP-READY REPO**: Este repositorio contiene definiciones locales de servidores MCP.
> Cualquier agente de IA DEBE cargar los servidores definidos en [mcpServers.json](file:///home/ruben/Workspace/defi/bot_trading/mcpServers.json) antes de proceder con tareas complejas.

## 0. AGENT AUTO-DISCOVERY
Si eres un agente de IA, inicializa tu contexto con estas herramientas locales:
- `chassis-solana`: Acceso RPC a Helius.
- `chassis-telemetry`: Consultas SQL a `trading_state.db`.
- `chassis-files`: Gestión de sistema de archivos local.
- `chassis-git`: Operaciones de versionado.

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
