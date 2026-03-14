---
applyTo: ["**/*.rs"]
excludeFrom: ["**/programs/**/*.rs"]
---

- Error handling: thiserror para libs, anyhow para bins. Sin unwrap() en lib
- Async: Tokio runtime. prefer spawn_blocking para CPU-bound
- Performance: prefer stack allocation, avoid Box<dyn> en hot paths
- Concurrencia: prefer channels (tokio::mpsc) sobre shared state
- Logging: tracing crate, structured logs, RUST_LOG en env
- Clippy: cargo clippy -- -D warnings sin excepciones
- Formato: cargo fmt obligatorio antes de commit
