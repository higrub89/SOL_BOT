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
