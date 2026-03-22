# GitHub Copilot Instructions — Ruben Antigravity Standard 2026

## Identidad del operador
- Ruben — HFT/DeFi en Solana
- Entorno: Ubuntu 24.04 LTS, ThinkPad L14 Gen1 32gm ram intel core i5
- Bytemuck zero-copy, no msg! en producción
- Bash: set -euo pipefail + shellcheck obligatorio

## Reglas obligatorias (tolerancia cero)
- Commits: Conventional Commits en inglés (feat/fix/perf/refactor/chore) SIEMPRE
- Secrets: fail-fast si !env var, nunca hardcode
- Sin deuda técnica: TODO + fecha + explicación si es inevitable
- Rust: thiserror + anyhow, prohibido unwrap() fuera de main/tests
- TypeScript: exhaustive switch, discriminated unions, no enums nativos
- Rebase estricto, prohibido merge commits en proyectos personales

## Alertas inmediatas (detener implementación)
- Memory leaks / buffer overflows
- Signer misuse / CPI sin validación
- Clock drift / reentrancy en contratos
- Secrets expuestos en cualquier forma
- Operaciones destructivas sin confirmación explícita

## Estilo
- Legibilidad > cleverness
- Comentarios solo cuando el código no se explica solo
- Tests: mínimo 80% coverage, proptest cuando aplique
- Zero-copy y minimal allocations en código HFT
