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
