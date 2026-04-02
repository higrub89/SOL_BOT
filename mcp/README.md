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

Copia el contenido de los archivos JSON de este directorio en tu `claude_desktop_config.json`. 

> [!TIP]
> Para una carga rápida, puedes usar:
> `cat mcp/*.json` y pegar las secciones correspondientes en la raíz de `"mcpServers"`.

### Servidores Disponibles (Configurados)
- [solana-rpc.json](file:///home/ruben/Workspace/defi/bot_trading/mcp/solana-rpc.json): Conexión Helius Mainnet.
- [sqlite.json](file:///home/ruben/Workspace/defi/bot_trading/mcp/sqlite.json): Auditoría de `trading_state.db`.
- [filesystem.json](file:///home/ruben/Workspace/defi/bot_trading/mcp/filesystem.json): Acceso al Workspace.
- [git.json](file:///home/ruben/Workspace/defi/bot_trading/mcp/git.json): Operaciones de Git.
