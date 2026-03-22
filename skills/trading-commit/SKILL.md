---
name: trading-commit
description: Protocolo estricto para Conventional Commits (Aerospace Standard).
auto-invokes:
  - Crear un git commit
  - Modificar o añadir features o fixes al log de git
---
# Trading Commit Standard

Sigue estrictamente el conventional-commit.
**Formato:** `<tipo>[scope opcional]: <descripción imperativa>`

**Tipos Permitidos:**
- `feat`: Nueva característica / Estrategia
- `fix`: Resolución de bug / PnL leak
- `perf`: Reducción de latencia u optimización extrema
- `refactor`: Limpieza que no afecta latencia ni lógica
- `docs`, `chore`, `test`

*El mensaje del commit debe estar en Inglés.*
Ejemplo de `bot_trading`:
`perf(core): pre-allocate arena for geyser logs parsing`
