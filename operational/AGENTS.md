# AGENTS.md — operational/ (Mission Ops)
> Overrides root AGENTS.md for all work inside operational/.

## ⚠️ Security — Read First
- `operational/wallets/` is NEVER accessible via MCP directly.
- Keys are NEVER logged, printed, or committed.
- All execute-critical operations must be piped through `trading-mcp-server` Zod validators.
- All audit entries require a Report ID.

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
