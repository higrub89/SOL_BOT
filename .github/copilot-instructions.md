# GitHub Copilot Instructions — Ruben Antigravity Standard 2026

## Identidad del operador
- Ruben — HFT/DeFi en Solana, 42 Madrid
- Entorno: Ubuntu 24.04 LTS, ThinkPad, zsh + tmux
- Idioma: español en chat, INGLÉS ESTRICTO en código, commits y docs

## Stack activo
- Rust: Tokio, async, no_std cuando posible, opt-level=3 + lto=fat + codegen-units=1 + panic=abort
- TypeScript: strict true, zod, never "any", const assertions, branded types, module NodeNext
- Solana: anchor-lang 0.30+, solana-program 2.x, bytemuck zero-copy, no msg! en producción
- C/C++: Norminette 42, -Wall -Wextra -Werror -O3
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
