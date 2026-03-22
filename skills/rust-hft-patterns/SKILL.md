---
name: rust-hft-patterns
description: Patrones de latencia extrema para el core de Trading (Zero-copy, lock-free, allocation-free).
auto-invokes:
  - Edit any .rs file in core/
  - Modifying the ExecutionRouter
---
# Rust HFT Patterns

## 1. Zero-Allocation Hot Paths
- **Regla de Oro:** Ninguna asignación al Heap (`Box`, `Vec`, `String`) en el hot path. Las instrucciones deben construirse stack-allocated o en buffers pre-reservados.
- **Deserialización Zero-Copy:** Usa `zerocopy` o punteros directos para mapear el estado de accounts de Solana.

## 2. Lock-free State
- Evitar `Arc<Mutex<T>>` / `Arc<RwLock<T>>` en paths de alta frecuencia.
- Prefiere Canales MPSC (`flume` o `crossbeam_channel`) o estructuras Atómicas (`AtomicU64`, `AtomicPtr`).

## 3. Manejo de Errores y Panics
- `unwrap()` está extrictamente **PROHIBIDO** en código de producción. Devuelve resultados propagando `?`.
- Para situaciones determinísticas ineludibles, usar macros seguras o aserciones estáticas de compilación.

## 4. Dependencias y Optimizaciones
- `Cargo.toml`: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`.
