# MCP Security Protocols (Aerospace Tier)

## ⚠️ DEPRECATION NOTICE
The use of static JSON configurations (`filesystem.json`, `git.json`, `solana-rpc.json`) in this directory for programmatic execution is **DEPRECATED**. 

## New Unified Standard
All interactions requiring execution boundaries (sending transactions, executing PANIC_ALL, reading core secrets) MUST go through the `trading-mcp-server`.

**Why?**
- **Zod Validation:** The MCP Server validates all payloads before hitting the Rust engine. An anomalous `panic_sell` with infinite slippage will crash at the Zod validation layer, not the blockchain execution layer.
- **Telemetry:** Allows tracing agent requests directly in the Node.js middleware.

## Future Action
The AI Agent must rely on `execute_trade_command` from the MCP Server and cease using `run_command` via arbitrary bash scripts for critical trading infra changes.
