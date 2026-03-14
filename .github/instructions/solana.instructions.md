---
applyTo: ["**/*.rs", "**/programs/**/*", "**/Anchor.toml", "**/Cargo.toml"]
---

- anchor-lang = "0.30+" y solana-program = "2.x" como baseline
- CPI: verificar signer y bumps en cada cross-program invocation
- Cuentas: prefer zero-copy con bytemuck, minimizar heap allocations
- Prohibido solana_program::msg! en producción — coste de compute units
- Seeds y PDAs: documentar derivación en comentario inline
- Error handling: define errores custom con #[error_code], nunca panic
- Tests: usar bankrun o solana-program-test, nunca solo devnet
