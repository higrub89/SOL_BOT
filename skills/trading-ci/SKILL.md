---
name: trading-ci
description: CI/CD checks de seguridad y regresión para repositorios HFT.
auto-invokes:
  - Configurar GitHub Actions o flujos automatizados (CI)
  - Evaluar impacto de performance
---
# Trading CI/CD Standards

1. **Cargo Checks:** Los pipelines deben rechazar compilaciones con warnings (`cargo clippy -- -D warnings`).
2. **Auditoría de Dependencias:** `cargo audit` se pasará antes de compilar y si falla, bloquea el pipeline (Zero-Trust vulnerabilities).
3. **Latencia / Regression:** Corre los benchmarks de *Criterion* (`cargo bench`). Si la latencia en milisegundos de construcción de transacción supera el threshold de (P50 < 0.2ms), el PR fracasa.
