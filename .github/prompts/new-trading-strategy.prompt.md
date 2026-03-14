---
description: Scaffold completo de nueva estrategia HFT para Solana
mode: agent
---

Crea una nueva estrategia de trading HFT en Rust para Solana:

1. `src/strategies/<nombre>.rs` — lógica principal
   - Struct con configuración, Trait Strategy implementado
   - Zero-copy donde aplique, sin unwrap(), manejo exhaustivo de errores

2. `src/strategies/mod.rs` — registrar la nueva estrategia

3. `tests/strategies/<nombre>_test.rs`
   - Tests unitarios con proptest para casos edge
   - Mock de RPC responses
   - Métricas: latencia estimada, slippage

4. Documentación inline: descripción, DEXs objetivo, condiciones entrada/salida, riesgos

No añadir dependencias sin justificación explícita.
