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
