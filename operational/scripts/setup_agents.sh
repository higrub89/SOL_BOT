#!/usr/bin/env bash
# ============================================================
#  BOT_TRADING — Agent Workspace Setup  v1.1.0 (Final)
#  Standard: Prowler-style agentskills.io
#  Run from: project root (~/bot_trading)
#
#  Refinements v1.1.0:
#    · ml-signal skill: IPC strategy tiers (ZMQ / UDS / SHM)
#    · Makefile: make clean-deep (cargo + python + logs)
#    · core/proto/signal.proto: Python↔Rust signal contract
# ============================================================
set -euo pipefail

GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BOLD='\033[1m'
NC='\033[0m'

info()    { echo -e "${CYAN}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC}   $1"; }
section() { echo -e "\n${BOLD}${YELLOW}▶ $1${NC}"; }
warn()    { echo -e "${RED}[WARN]${NC} $1"; }

# ──────────────────────────────────────────────
section "0 · Pre-flight — Backup existing files"
# ──────────────────────────────────────────────
BACKUP_DIR=".backup_pre_agents_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$BACKUP_DIR"

FILES_TO_BACKUP=("Makefile" ".gitignore" ".env.example" "AGENTS.md")
for f in "${FILES_TO_BACKUP[@]}"; do
  if [ -f "$f" ]; then
    cp "$f" "$BACKUP_DIR/$f"
    warn "Backed up: $f → $BACKUP_DIR/$f"
  fi
done
success "Backup complete → $BACKUP_DIR/"
echo -e "  ${CYAN}To restore:${NC} cp $BACKUP_DIR/Makefile . && cp $BACKUP_DIR/.gitignore ."


# ──────────────────────────────────────────────
section "1 · Root AGENTS.md — The Mission Control"
# ──────────────────────────────────────────────
cat > AGENTS.md << 'EOF'
# AGENTS.md — Bot_Trading Mission Control
> Start here. This file is the single source of truth for any AI agent working on this project.
> Each subdirectory has its own AGENTS.md that overrides this file when guidance conflicts.

---

## Project Overview

**Codename:** The Chassis
**Mission:** High-frequency Solana trading bot with ML-driven signal generation.
**Lead Engineer:** Rubén
**Stack:** Rust (core engine) · Python (intelligence/ML) · Solana mainnet-beta

### Critical Path
```
Geyser/RPC Feed → Sensors → Signal Engine → Execution → Telegram Alerts
```

---

## Repository Structure

| Directory       | Purpose                                         | Language |
|-----------------|-------------------------------------------------|----------|
| `core/`         | Low-latency execution engine (The Chassis)      | Rust     |
| `intelligence/` | ML models, datasets, signal research            | Python   |
| `operational/`  | Wallets, audits, DevOps scripts                 | Shell    |
| `docs/`         | BLUE_BOOK — architecture & spec                 | Markdown |
| `skills/`       | AI agent skills (agentskills.io standard)       | Markdown |
| `mcp/`          | MCP server configurations                       | JSON     |

---

## Available Skills

| Skill                    | File                                        | When to Use                              |
|--------------------------|---------------------------------------------|------------------------------------------|
| `solana-trading`         | `skills/solana-trading/SKILL.md`           | Any Solana RPC/transaction work          |
| `rust-engine`            | `skills/rust-engine/SKILL.md`              | Editing code in `core/`                  |
| `wallet-ops`             | `skills/wallet-ops/SKILL.md`               | Wallet, keypair, vault operations        |
| `trading-audit`          | `skills/trading-audit/SKILL.md`            | Writing audit reports in `operational/` |
| `ml-signal`              | `skills/ml-signal/SKILL.md`               | ML models and signal research            |
| `mcp-config`             | `skills/mcp-config/SKILL.md`              | Configuring or extending MCPs            |

### Auto-invoke Skills
When performing these actions, **ALWAYS** invoke the corresponding skill **FIRST**:

| Action                                          | Skill            |
|-------------------------------------------------|------------------|
| Edit any `.rs` file in `core/`                  | `rust-engine`    |
| Call Solana RPC, send transactions, parse slots | `solana-trading` |
| Create or rotate a wallet / keypair             | `wallet-ops`     |
| Write to `operational/audits/`                  | `trading-audit`  |
| Modify `intelligence/` models or datasets       | `ml-signal`      |
| Edit `mcp/*.json` or add a new MCP server       | `mcp-config`     |

---

## MCP Servers

| Server           | Config File                  | Scope                                    |
|------------------|------------------------------|------------------------------------------|
| `filesystem`     | `mcp/filesystem.json`        | Read/Write `core/`, `operational/`       |
| `git`            | `mcp/git.json`               | Commits, branches, diff                  |
| `memory`         | `mcp/memory.json`            | Persistent session context               |
| `solana-rpc`     | `mcp/solana-rpc.json`        | Mainnet-beta & devnet RPC calls          |

---

## Quality Standards (Non-Negotiable)

- All Rust code must pass `cargo clippy -- -D warnings` and `cargo audit`
- No `unwrap()` in production paths — use `?` or explicit error handling
- Wallet keys are **never** hardcoded — use `operational/wallets/` vault pattern
- Every trade execution must be logged to `operational/logs/`
- ML models require a validation report in `intelligence/datasets/` before merge

---

## Setup & Commands

```bash
# Build core engine
cd core && cargo build --release

# Run linters
cargo clippy -- -D warnings
cargo fmt --check

# Load AI skills (agentskills.io)
./skills/setup.sh

# Audit security
cargo audit
```

---

## Component AGENTS.md Files

- [`core/AGENTS.md`](core/AGENTS.md) — Rust engine patterns, module ownership
- [`intelligence/AGENTS.md`](intelligence/AGENTS.md) — ML workflows, dataset conventions
- [`operational/AGENTS.md`](operational/AGENTS.md) — Wallet security, audit format
EOF
success "AGENTS.md created"

# ──────────────────────────────────────────────
section "2 · Skills Directory"
# ──────────────────────────────────────────────
mkdir -p skills/{solana-trading,rust-engine,wallet-ops,trading-audit,ml-signal,mcp-config}

# skills/README.md
cat > skills/README.md << 'EOF'
# Skills — Bot_Trading Agent Intelligence Layer

Skills are structured instruction files following the [agentskills.io](https://agentskills.io) standard.
Each skill teaches an AI agent how to work within this specific project's conventions.

## Structure

```
skills/
├── {skill-name}/
│   ├── SKILL.md       # Required — instructions and metadata
│   ├── scripts/       # Optional — helper scripts
│   └── assets/        # Optional — templates, schemas
└── setup.sh           # Configures skills for each AI tool
```

## Available Skills

| Skill             | Description                                       |
|-------------------|---------------------------------------------------|
| `solana-trading`  | Solana RPC, transactions, and slot telemetry      |
| `rust-engine`     | Rust patterns for the core execution engine       |
| `wallet-ops`      | Secure wallet and keypair management              |
| `trading-audit`   | Audit log format and compliance reporting         |
| `ml-signal`       | ML model conventions and signal validation        |
| `mcp-config`      | MCP server setup and extension patterns           |

## Setup

```bash
./skills/setup.sh
```

This symlinks skills for Claude Code, Cursor, Codex, and GitHub Copilot.
EOF

# setup.sh
cat > skills/setup.sh << 'SETUPEOF'
#!/usr/bin/env bash
# Configures skills for AI coding assistants (agentskills.io standard)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

echo "🔧 Setting up Agent Skills for bot_trading..."

tools=(
  ".claude/skills"
  ".codex/skills"
  ".github/skills"
  ".gemini/skills"
)

for tool_dir in "${tools[@]}"; do
  full_path="$ROOT_DIR/$tool_dir"
  mkdir -p "$(dirname "$full_path")"
  if [ -L "$full_path" ]; then
    rm "$full_path"
  fi
  ln -s "$SCRIPT_DIR" "$full_path"
  echo "  ✓ $tool_dir → skills/"
done

echo ""
echo "✅ Skills configured. Restart your AI coding assistant to load them."
echo "   Gemini CLI: enable 'experimental.skills' in settings."
SETUPEOF
chmod +x skills/setup.sh
success "skills/setup.sh created"

# ── SKILL: solana-trading ──────────────────────
cat > skills/solana-trading/SKILL.md << 'EOF'
---
name: solana-trading
description: >
  Use this skill for any Solana-specific work: RPC calls, transaction building,
  account parsing, slot telemetry via Geyser, Raydium/DEX interaction, or
  anything in core/src/sensors/ and core/src/engine/.
metadata:
  scope:
    - core/src/sensors/
    - core/src/engine/
    - core/src/bin/
  auto_invoke:
    - "Call Solana RPC"
    - "Parse account data"
    - "Build or send a transaction"
    - "Subscribe to Geyser slot stream"
    - "Interact with Raydium or Jupiter"
---

# Skill: Solana Trading Engine

## RPC Connection Pattern

Always use the **connection pool** in `core/src/engine/rpc.rs`. Never create ad-hoc RPC clients.

```rust
// ✅ Correct
let rpc = ctx.rpc_pool.get_client(Commitment::Confirmed)?;

// ❌ Wrong — creates latency spikes
let rpc = RpcClient::new("https://api.mainnet-beta.solana.com");
```

## Slot Telemetry (Geyser)

Geyser subscriptions live in `core/src/sensors/geyser.rs`. Use the `SlotSensor` trait:

```rust
impl SlotSensor for MyMonitor {
    async fn on_slot(&self, slot: Slot, parent: Slot) -> Result<()> {
        // telemetry logic here
    }
}
```

## Transaction Building

- Always set `compute_unit_limit` and `compute_unit_price` explicitly
- Use `VersionedTransaction` (v0) for Address Lookup Tables
- Maximum retry: 3 attempts with exponential backoff (50ms, 100ms, 200ms)

## Error Handling

All RPC errors must map to the project's `ChassisError` enum in `core/src/engine/error.rs`.
Never surface raw `solana_client::client_error::ClientError` to callers.

## Testing

Integration tests use devnet. Tag them with `#[ignore]` so CI skips them:
```rust
#[tokio::test]
#[ignore = "requires devnet connection"]
async fn test_slot_subscription() { ... }
```
EOF
success "skills/solana-trading/SKILL.md"

# ── SKILL: rust-engine ──────────────────────
cat > skills/rust-engine/SKILL.md << 'EOF'
---
name: rust-engine
description: >
  Use this skill when editing any Rust source file in core/. Covers module
  architecture, error handling patterns, async conventions, and performance
  guidelines for the low-latency execution engine.
metadata:
  scope:
    - core/src/
    - core/proto/
  auto_invoke:
    - "Edit .rs file"
    - "Add new module to core"
    - "Implement a trait"
    - "Fix a Rust compile error"
---

# Skill: Rust Engine Patterns

## Module Ownership

```
core/src/
├── bin/          → Executable entry points (one binary per concern)
├── engine/       → Execution logic: order routing, risk, position mgmt
├── sensors/      → Data ingestion: Geyser, RPC polling, price feeds
├── telegram/     → Notification layer (commands/, handlers)
└── generated/    → Protobuf output — DO NOT edit manually
```

## Error Handling

Use `thiserror` for all error types. Every public function returns `Result<T, ChassisError>`.

```rust
// ✅ Good
#[derive(thiserror::Error, Debug)]
pub enum ChassisError {
    #[error("RPC connection failed: {0}")]
    Rpc(#[from] solana_client::client_error::ClientError),

    #[error("Insufficient balance: need {needed}, have {available}")]
    InsufficientBalance { needed: u64, available: u64 },
}
```

## Async Conventions

- Runtime: `tokio` with `#[tokio::main]`
- Prefer `tokio::spawn` over `std::thread` for IO-bound work
- CPU-bound work → `rayon` thread pool, not `tokio::spawn_blocking`
- All channel sizes must be explicit: `tokio::sync::mpsc::channel(1024)`

## Performance Rules

1. Zero allocations in hot paths (slot processing, order book updates)
2. Use `Arc<T>` for shared state, `Mutex<T>` only when necessary
3. Pre-allocate `Vec` with known capacity in sensor loops
4. Profile before optimizing — instrument with `tracing::instrument`

## Clippy Gates

```bash
cargo clippy -- -D warnings -D clippy::pedantic -A clippy::module_name_repetitions
```

All CI merges must pass this. Fix warnings, never `#[allow]` them.
EOF
success "skills/rust-engine/SKILL.md"

# ── SKILL: wallet-ops ──────────────────────
cat > skills/wallet-ops/SKILL.md << 'EOF'
---
name: wallet-ops
description: >
  Use this skill for any wallet, keypair, or vault operation. Covers key
  generation, rotation, encrypted storage, and the operational/wallets/
  directory conventions.
metadata:
  scope:
    - operational/wallets/
  auto_invoke:
    - "Generate a keypair"
    - "Rotate a wallet"
    - "Access a private key"
    - "Manage vault"
---

# Skill: Wallet Operations

## Security Rules (Absolute)

1. **Private keys are NEVER hardcoded** — not in source, not in configs, not in tests
2. **Never commit** `operational/wallets/` to git — confirm `.gitignore` covers `*.json` and `*.key`
3. All vault access requires environment variable: `CHASSIS_VAULT_PASSWORD`
4. Production keypairs use hardware-backed storage when possible

## Key Generation

```bash
# Generate a new Solana keypair
solana-keygen new --outfile operational/wallets/trader_$(date +%Y%m%d).json

# Verify
solana-keygen pubkey operational/wallets/trader_YYYYMMDD.json
```

## Vault Directory Structure

```
operational/wallets/
├── .gitignore          # ← must contain: *.json, *.key, *.pem
├── README.md           # Documents wallet purposes (no keys)
└── {name}_{date}.json  # Encrypted keypair files
```

## Environment Loading

```rust
// Load keypair from env path — never inline
let keypair_path = std::env::var("CHASSIS_KEYPAIR_PATH")
    .expect("CHASSIS_KEYPAIR_PATH must be set");
let keypair = read_keypair_file(&keypair_path)
    .expect("Invalid keypair file");
```

## Rotation Checklist

- [ ] Generate new keypair
- [ ] Transfer minimal SOL for gas
- [ ] Update `CHASSIS_KEYPAIR_PATH` in environment
- [ ] Log rotation in `operational/audits/` with date and pubkey (no private key)
- [ ] Revoke old keypair access from any APIs
EOF
success "skills/wallet-ops/SKILL.md"

# ── SKILL: trading-audit ──────────────────────
cat > skills/trading-audit/SKILL.md << 'EOF'
---
name: trading-audit
description: >
  Use this skill when writing audit reports, trade logs, or compliance records
  in operational/audits/. Enforces consistent format for all financial records.
metadata:
  scope:
    - operational/audits/
    - operational/logs/
  auto_invoke:
    - "Write an audit report"
    - "Log a trade"
    - "Create compliance record"
    - "Document a wallet rotation"
---

# Skill: Trading Audit Format

## Audit Report Structure

All reports in `operational/audits/` follow this format:

```markdown
# Audit: {TYPE} — {YYYY-MM-DD}

**Report ID:** AUDIT-{YYYYMMDD}-{SEQ}
**Auditor:** {name or "automated"}
**Status:** PASS | FAIL | REVIEW

## Summary
One-paragraph executive summary.

## Findings
| # | Finding | Severity | Status |
|---|---------|----------|--------|
| 1 | ...     | LOW/MED/HIGH/CRITICAL | OPEN/RESOLVED |

## Evidence
Links to logs, transactions, or code references.

## Recommendations
Actionable next steps.
```

## Trade Log Format (`operational/logs/`)

Each line in trade logs is JSON:
```json
{
  "ts": "2025-01-01T00:00:00.000Z",
  "type": "BUY|SELL",
  "pair": "SOL/USDC",
  "amount": 1.5,
  "price": 180.42,
  "tx": "5xyz...abc",
  "slot": 301234567,
  "pnl_usd": null
}
```

## Naming Convention

- Audits: `operational/audits/AUDIT-{YYYYMMDD}-{SEQ}-{TYPE}.md`
- Logs: `operational/logs/{YYYY}/{MM}/trades_{YYYYMMDD}.jsonl`
EOF
success "skills/trading-audit/SKILL.md"

# ── SKILL: ml-signal ──────────────────────
cat > skills/ml-signal/SKILL.md << 'EOF'
---
name: ml-signal
description: >
  Use this skill when working in intelligence/: ML model training, signal
  validation, dataset management, or research notebooks.
metadata:
  scope:
    - intelligence/
  auto_invoke:
    - "Train a model"
    - "Validate a signal"
    - "Add to dataset"
    - "Run ML script"
    - "Connect Python to Rust engine"
---

# Skill: ML Signal Research

## Intelligence Directory Structure

```
intelligence/
├── datasets/
│   ├── raw/          → Unprocessed data (never modify)
│   └── processed/    → Feature-engineered, ready for training
├── ml/
│   ├── models/       → Saved model weights (.pt, .onnx)
│   └── scripts/
│       └── operational/audits/  → Model performance reports
├── scripts/          → Data collection, feature engineering
└── src/              → Library code shared across scripts
```

## Model Validation Gate

Before merging any model change, create a validation report in
`intelligence/ml/scripts/operational/audits/`:

```markdown
# Model Validation: {model_name} — {date}

**Sharpe Ratio:** ≥ 1.5 required
**Max Drawdown:** ≤ 15% required
**Win Rate:** ≥ 52% required
**Backtest Period:** minimum 90 days
**Dataset:** describe source and date range
```

---

## Signal → Engine IPC Strategy

The Python intelligence layer communicates with the Rust engine via
serialized Protobuf messages (`core/proto/signal.proto`).

**Choose the IPC transport based on observed latency budget:**

### Tier 1 — ZMQ TCP (Default · ~50–200µs localhost)
Use for initial development and when processes may run on different hosts.

```python
import zmq, time
from core.generated import signal_pb2

ctx = zmq.Context()
socket = ctx.socket(zmq.PUSH)
socket.connect("tcp://127.0.0.1:5555")   # env: ZMQ_SIGNAL_SOCKET

signal = signal_pb2.Signal(
    pair="SOL/USDC",
    direction=signal_pb2.Direction.LONG,
    confidence=0.87,
    timestamp_ms=int(time.time() * 1000),
)
socket.send(signal.SerializeToString())
```

### Tier 2 — Unix Domain Sockets (UDS · ~5–20µs)
Drop-in replacement when Python and Rust run on the same machine.
Eliminates TCP stack overhead with no code changes beyond the address:

```python
socket.connect("ipc:///tmp/chassis_signal.sock")   # ZMQ IPC transport
```

```rust
// Rust side — same change
let socket_addr = std::env::var("CHASSIS_IPC_SOCKET")
    .unwrap_or_else(|_| "ipc:///tmp/chassis_signal.sock".into());
```

### Tier 3 — Shared Memory / `mmap` (· ~500ns–2µs)
Use only when Tier 2 latency is still a bottleneck (measured, not assumed).
Requires a lock-free ring buffer. Reference implementation:
`core/src/sensors/shm_receiver.rs` (create when needed).

```
Python writer → mmap ring buffer → Rust reader
              ↑ lock-free, cache-line aligned
```

**Decision rule:** measure first with `cargo bench` + `hyperfine`.
Never migrate to a higher tier without a benchmark proving the gain.

---

## IPC Tier Environment Variables

```bash
# .env
ZMQ_SIGNAL_SOCKET=tcp://127.0.0.1:5555   # Tier 1
# ZMQ_SIGNAL_SOCKET=ipc:///tmp/chassis_signal.sock  # Tier 2
CHASSIS_IPC_MODE=zmq   # zmq | uds | shm
```
EOF
success "skills/ml-signal/SKILL.md"

# ── SKILL: mcp-config ──────────────────────
cat > skills/mcp-config/SKILL.md << 'EOF'
---
name: mcp-config
description: >
  Use this skill when configuring, extending, or debugging MCP (Model Context
  Protocol) server integrations in the mcp/ directory.
metadata:
  scope:
    - mcp/
  auto_invoke:
    - "Configure MCP server"
    - "Add new MCP"
    - "Debug MCP connection"
    - "Edit mcp/*.json"
---

# Skill: MCP Configuration

## Active MCP Servers

| Server       | File                   | Purpose                              |
|--------------|------------------------|--------------------------------------|
| filesystem   | `mcp/filesystem.json`  | File read/write for core & ops       |
| git          | `mcp/git.json`         | Version control operations           |
| memory       | `mcp/memory.json`      | Persistent agent session context     |
| solana-rpc   | `mcp/solana-rpc.json`  | Solana blockchain interactions       |

## Adding a New MCP Server

1. Create `mcp/{name}.json` following the existing schema
2. Add entry to the MCP table in root `AGENTS.md`
3. Document the scope (which directories/tools it covers)
4. Test with: `npx @modelcontextprotocol/inspector mcp/{name}.json`

## Filesystem MCP — Allowed Paths

The filesystem MCP is scoped to:
- `core/src/` — read/write
- `operational/audits/` — write only
- `operational/logs/` — write only
- `docs/` — read/write
- `intelligence/` — read/write

**Never grant filesystem access to `operational/wallets/`** via MCP.
EOF
success "skills/mcp-config/SKILL.md"

# ──────────────────────────────────────────────
section "3 · MCP Configurations"
# ──────────────────────────────────────────────
mkdir -p mcp

cat > mcp/README.md << 'EOF'
# MCP — Model Context Protocol Configurations

Each JSON file configures one MCP server for AI agent use.
See `AGENTS.md` for the full MCP table and access scopes.

## Quick Reference

| File               | Server            | Restart Required |
|--------------------|-------------------|------------------|
| `filesystem.json`  | Filesystem access | Yes              |
| `git.json`         | Git operations    | Yes              |
| `memory.json`      | Session memory    | No               |
| `solana-rpc.json`  | Solana RPC        | Yes              |

## Loading in Claude Desktop / Claude Code

Add the contents of each file to your `claude_desktop_config.json`:
```json
{
  "mcpServers": {
    "filesystem": { ... },
    "git": { ... }
  }
}
```
EOF

cat > mcp/filesystem.json << 'EOF'
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": [
        "-y",
        "@modelcontextprotocol/server-filesystem",
        "./core/src",
        "./operational/audits",
        "./operational/logs",
        "./docs",
        "./intelligence",
        "./skills"
      ],
      "_comment": "NEVER add operational/wallets/ to allowed paths"
    }
  }
}
EOF

cat > mcp/git.json << 'EOF'
{
  "mcpServers": {
    "git": {
      "command": "npx",
      "args": [
        "-y",
        "@modelcontextprotocol/server-git",
        "--repository",
        "."
      ]
    }
  }
}
EOF

cat > mcp/memory.json << 'EOF'
{
  "mcpServers": {
    "memory": {
      "command": "npx",
      "args": [
        "-y",
        "@modelcontextprotocol/server-memory"
      ]
    }
  }
}
EOF

cat > mcp/solana-rpc.json << 'EOF'
{
  "mcpServers": {
    "solana-rpc": {
      "command": "npx",
      "args": [
        "-y",
        "@solana/mcp-server"
      ],
      "env": {
        "SOLANA_CLUSTER": "mainnet-beta",
        "SOLANA_RPC_URL": "${SOLANA_RPC_URL}",
        "SOLANA_DEVNET_URL": "https://api.devnet.solana.com"
      },
      "_comment": "Set SOLANA_RPC_URL in your .env — never hardcode"
    }
  }
}
EOF
success "mcp/ directory configured"

# ──────────────────────────────────────────────
section "4 · Component AGENTS.md Files"
# ──────────────────────────────────────────────
mkdir -p core intelligence operational

cat > core/AGENTS.md << 'EOF'
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
| Edit any `.rs` file           | `rust-engine`    |
| Work with Geyser / RPC / DEX  | `solana-trading` |

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
EOF

cat > intelligence/AGENTS.md << 'EOF'
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
EOF

cat > operational/AGENTS.md << 'EOF'
# AGENTS.md — operational/ (Mission Ops)
> Overrides root AGENTS.md for all work inside operational/.

## ⚠️ Security — Read First
- `operational/wallets/` is NEVER accessible via MCP
- Keys are NEVER logged, printed, or committed
- All audit entries require a Report ID

## Auto-invoke Skills
| Action                          | Skill           |
|---------------------------------|-----------------|
| Write to audits/ or logs/       | `trading-audit` |
| Generate or rotate a keypair    | `wallet-ops`    |

## Directory Rules
```
operational/
├── wallets/   → NEVER in MCP scope. NEVER committed. Encrypted only.
├── audits/    → Audit reports (AUDIT-YYYYMMDD-SEQ-TYPE.md)
├── logs/      → Trade logs (YYYY/MM/trades_YYYYMMDD.jsonl)
└── scripts/   → DevOps automation
```
EOF
success "Component AGENTS.md files created"

# ──────────────────────────────────────────────
section "5 · .gitignore (Calidad Suiza)"
# ──────────────────────────────────────────────
cat > .gitignore << 'EOF'
# ── Build artifacts ────────────────────────────────────────
/target/
**/*.rs.bk
Cargo.lock.bak

# ── Secrets & Wallets (CRITICAL) ───────────────────────────
operational/wallets/*.json
operational/wallets/*.key
operational/wallets/*.pem
.env
.env.*
!.env.example

# ── Runtime databases & caches ──────────────────────────────
trading_state.db
trading_state.db-shm
trading_state.db-wal
pools_cache.json
*.db
*.db-shm
*.db-wal

# ── Logs & Runtime data ─────────────────────────────────────
operational/logs/
logs/

# ── Intelligence / ML artifacts ─────────────────────────────
intelligence/ml/models/*.pt
intelligence/ml/models/*.onnx
intelligence/datasets/raw/
__pycache__/
*.pyc
*.pyo
.venv/
.python-version

# ── IDE & OS ────────────────────────────────────────────────
.idea/
.vscode/settings.json
*.swp
*.swo
.DS_Store
Thumbs.db

# ── Agent tool symlinks (generated by skills/setup.sh) ──────
.claude/
.codex/
.gemini/
# .github/skills is committed — remove from ignore if needed

# ── Setup backup dirs ────────────────────────────────────────
.backup_pre_agents_*/
EOF
success ".gitignore updated"

# ──────────────────────────────────────────────
section "6 · .env.example (Safe template — merge mode)"
# ──────────────────────────────────────────────
# Merge: only add variables that don't already exist
ENV_EXAMPLE=".env.example"
add_env_var() {
  local key="$1" line="$2"
  if ! grep -q "^${key}=" "$ENV_EXAMPLE" 2>/dev/null; then
    echo "$line" >> "$ENV_EXAMPLE"
    info "Added to .env.example: $key"
  fi
}

# Create if it doesn't exist
touch "$ENV_EXAMPLE"

# Ensure section headers and new variables are present
add_env_var "SOLANA_RPC_URL"        "SOLANA_RPC_URL=https://your-rpc-provider.com/your-api-key"
add_env_var "SOLANA_CLUSTER"        "SOLANA_CLUSTER=mainnet-beta"
add_env_var "CHASSIS_KEYPAIR_PATH"  "CHASSIS_KEYPAIR_PATH=operational/wallets/trader_YYYYMMDD.json"
add_env_var "CHASSIS_VAULT_PASSWORD" "CHASSIS_VAULT_PASSWORD="
add_env_var "TELEGRAM_BOT_TOKEN"    "TELEGRAM_BOT_TOKEN="
add_env_var "TELEGRAM_CHAT_ID"      "TELEGRAM_CHAT_ID="
add_env_var "ZMQ_SIGNAL_SOCKET"     "ZMQ_SIGNAL_SOCKET=tcp://127.0.0.1:5555"
add_env_var "CHASSIS_IPC_MODE"      "CHASSIS_IPC_MODE=zmq   # zmq | uds | shm"
success ".env.example updated (merge mode — existing vars preserved)"
success ".env.example created"

# ──────────────────────────────────────────────
section "7 · Makefile (Command Center)"
# ──────────────────────────────────────────────
cat > Makefile << 'EOF'
.PHONY: all build release lint test audit clean clean-deep proto agents help

# ── Default ──────────────────────────────────────────────────
all: build

# ── Build ────────────────────────────────────────────────────
build:
	cd core && cargo build

release:
	cd core && cargo build --release

# ── Quality Gates ────────────────────────────────────────────
lint:
	cd core && cargo clippy -- -D warnings
	cd core && cargo fmt --check

test:
	cd core && cargo test

audit:
	cd core && cargo audit

# ── Protobuf ─────────────────────────────────────────────────
proto:
	@echo "🔧 Regenerating Protobuf bindings..."
	cd core && cargo build  # triggers build.rs
	@echo "   Rust bindings → core/src/generated/"
	@if command -v python3 >/dev/null 2>&1; then \
	  python3 -m grpc_tools.protoc \
	    -I core/proto \
	    --python_out=intelligence/src \
	    --pyi_out=intelligence/src \
	    core/proto/signal.proto && \
	  echo "   Python bindings → intelligence/src/"; \
	else \
	  echo "   [SKIP] grpc_tools not found — pip install grpcio-tools"; \
	fi

# ── Agent Workspace ──────────────────────────────────────────
agents:
	@echo "🤖 Setting up AI agent skills..."
	./skills/setup.sh

# ── Cleanup ──────────────────────────────────────────────────
clean:
	cd core && cargo clean

clean-deep: clean
	@echo "🧹 Deep clean — removing all build caches and runtime artifacts..."
	@# Python bytecode caches
	find . -type d -name "__pycache__" -not -path "./.git/*" -exec rm -rf {} + 2>/dev/null || true
	find . -name "*.pyc" -o -name "*.pyo" -not -path "./.git/*" -delete 2>/dev/null || true
	@# Python virtual env artifacts
	find . -name ".pytest_cache" -not -path "./.git/*" -exec rm -rf {} + 2>/dev/null || true
	find . -name "*.egg-info" -not -path "./.git/*" -exec rm -rf {} + 2>/dev/null || true
	@# Residual log files (preserve directory structure)
	find ./operational/logs -name "*.log" -o -name "*.jsonl" -delete 2>/dev/null || true
	find ./logs -name "*.log" -delete 2>/dev/null || true
	@# Rust incremental compile cache
	find ./core/target -name "incremental" -type d -exec rm -rf {} + 2>/dev/null || true
	@echo "   ✓ Python caches cleared"
	@echo "   ✓ Log residuals cleared"
	@echo "   ✓ Incremental Rust cache cleared"
	@echo "   Done. Run 'make build' for a clean compile."

# ── Help ─────────────────────────────────────────────────────
help:
	@echo ""
	@echo "  Bot_Trading — The Chassis"
	@echo ""
	@echo "  ── Build ───────────────────────────────────"
	@echo "  make build       → Debug build"
	@echo "  make release     → Optimized release build"
	@echo "  make proto       → Regenerate Protobuf bindings (Rust + Python)"
	@echo ""
	@echo "  ── Quality Gates ───────────────────────────"
	@echo "  make lint        → Clippy + fmt check"
	@echo "  make test        → Run test suite"
	@echo "  make audit       → Cargo security audit"
	@echo ""
	@echo "  ── Agent Workspace ─────────────────────────"
	@echo "  make agents      → Setup AI skills for all tools"
	@echo ""
	@echo "  ── Cleanup ─────────────────────────────────"
	@echo "  make clean       → Remove Rust build artifacts"
	@echo "  make clean-deep  → Clean everything (Rust + Python + logs)"
	@echo ""
EOF
success "Makefile created"

# ──────────────────────────────────────────────
section "8 · core/proto/signal.proto — Python↔Rust Contract"
# ──────────────────────────────────────────────
mkdir -p core/proto

cat > core/proto/signal.proto << 'EOF'
// signal.proto — The Chassis Signal Contract
// Python intelligence layer → Rust execution engine
//
// Regenerate bindings:
//   make proto
//
// Rust:  core/src/generated/  (via prost + build.rs)
// Python: intelligence/src/signal_pb2.py (via grpcio-tools)

syntax = "proto3";

package chassis;

// ─── Enums ─────────────────────────────────────────────────

enum Direction {
  DIRECTION_UNSPECIFIED = 0;
  LONG  = 1;
  SHORT = 2;
  EXIT  = 3;   // Close existing position
}

enum SignalStatus {
  STATUS_UNSPECIFIED = 0;
  PENDING   = 1;   // Generated, not yet consumed
  EXECUTING = 2;   // Engine is acting on it
  FILLED    = 3;   // Order confirmed on-chain
  REJECTED  = 4;   // Engine rejected (risk/balance)
  EXPIRED   = 5;   // TTL exceeded before execution
}

enum SignalSource {
  SOURCE_UNSPECIFIED = 0;
  ML_MODEL    = 1;   // intelligence/ model output
  RULE_BASED  = 2;   // Deterministic strategy
  MANUAL      = 3;   // Operator override via Telegram
}

// ─── Core Messages ─────────────────────────────────────────

// Signal: primary message from Python → Rust
message Signal {
  // Identity
  string  signal_id     = 1;   // UUID v4, set by Python
  string  model_version = 2;   // e.g. "v1.3.2" for audit trail

  // Market
  string  pair          = 3;   // e.g. "SOL/USDC"
  string  base_mint     = 4;   // Solana mint address of base token
  string  quote_mint    = 5;   // Solana mint address of quote token

  // Signal content
  Direction    direction   = 6;
  SignalSource source      = 7;
  double       confidence  = 8;   // 0.0–1.0; reject if < threshold
  double       entry_price = 9;   // Suggested entry (informational)
  double       size_usd    = 10;  // Suggested position size in USD

  // Risk parameters (optional; engine may override)
  double stop_loss_pct   = 11;   // e.g. 0.02 = 2%
  double take_profit_pct = 12;   // e.g. 0.04 = 4%

  // Time
  uint64 timestamp_ms    = 13;   // Unix epoch ms (Python time.time()*1000)
  uint32 ttl_ms          = 14;   // Signal expires after N ms (default: 5000)

  // Metadata
  map<string, string> tags = 15; // Arbitrary key-value for debugging
}

// Acknowledgement: Rust engine → Python (optional feedback loop)
message SignalAck {
  string        signal_id  = 1;
  SignalStatus  status     = 2;
  string        tx_sig     = 3;   // Solana tx signature if filled
  uint64        slot       = 4;   // Slot at execution
  uint64        ack_ts_ms  = 5;   // Engine timestamp
  string        reject_reason = 6; // Populated if status == REJECTED
}

// Heartbeat: Python → Rust, proves the signal process is alive
message Heartbeat {
  string  process_id   = 1;
  string  model_version = 2;
  uint64  timestamp_ms = 3;
  uint32  signals_sent = 4;   // Counter since process start
}
EOF
success "core/proto/signal.proto created"

# ──────────────────────────────────────────────
section "9 · Wallet safety net"
# ──────────────────────────────────────────────
mkdir -p operational/wallets
cat > operational/wallets/.gitignore << 'EOF'
# This directory contains encrypted keypairs.
# NOTHING in this directory should ever be committed.
*
!.gitignore
!README.md
EOF

cat > operational/wallets/README.md << 'EOF'
# Wallets — Secure Keypair Storage

This directory stores encrypted Solana keypairs.

## Rules
- Files NEVER leave this machine unencrypted
- Naming: `{purpose}_{YYYYMMDD}.json`  (e.g., `trader_20250322.json`)
- Before creating a wallet, read: `skills/wallet-ops/SKILL.md`

## Current Wallets
| File | Purpose | Pubkey | Created |
|------|---------|--------|---------|
| (add entries here — no private keys) | | | |
EOF
success "operational/wallets/ safety net set"

# ──────────────────────────────────────────────
echo ""
echo -e "${BOLD}${GREEN}════════════════════════════════════════════${NC}"
echo -e "${BOLD}${GREEN}  ✅  Agent Workspace Ready — The Chassis v1.1.0${NC}"
echo -e "${BOLD}${GREEN}════════════════════════════════════════════${NC}"
echo ""
echo -e "  ${CYAN}Run this in your bot_trading root:${NC}"
echo ""
echo -e "    ${BOLD}make agents${NC}       → Link skills (Claude Code / Cursor / Codex)"
echo -e "    ${BOLD}make proto${NC}        → Generate Rust + Python Protobuf bindings"
echo -e "    ${BOLD}make lint${NC}         → Quality gate"
echo -e "    ${BOLD}make clean-deep${NC}   → Full workspace clean"
echo ""
echo -e "  ${YELLOW}Files created:${NC}"
echo "    AGENTS.md                         ← Root mission control"
echo "    core/proto/signal.proto           ← Python↔Rust signal contract"
echo "    skills/{6 skills}/SKILL.md        ← Agent intelligence layer"
echo "    skills/ml-signal: IPC tiers       ← ZMQ → UDS → SHM strategy"
echo "    mcp/{4 servers}.json              ← MCP configurations"
echo "    core/AGENTS.md                    ← Rust engine context"
echo "    intelligence/AGENTS.md            ← ML context"
echo "    operational/AGENTS.md             ← Ops security context"
echo "    Makefile (+ clean-deep + proto)   ← Command center"
echo "    .gitignore                        ← Clean + secure"
echo "    .env.example                      ← Safe env template"
echo ""
echo -e "  ${CYAN}First test — paste this in your AI agent:${NC}"
echo '    "Read AGENTS.md and summarize my mission"'
echo ""
echo -e "  ${YELLOW}⚠️  Manual checks required:${NC}"
echo "    1. Review settings.json — if contains secrets, add to .gitignore"
echo "    2. Review your existing Makefile in .backup_pre_agents_*/"
echo "       Merge any custom targets into the new one"
echo "    3. Run: git status — confirm trading_state.db is now ignored"
echo "    4. Run: make agents — link skills to your AI tools"
echo ""