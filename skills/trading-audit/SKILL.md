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
