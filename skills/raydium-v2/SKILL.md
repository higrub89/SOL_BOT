---
name: raydium-v2
description: Hyper-optimized integration patterns for Raydium AMM V2/V4 on Solana (Zero-alloc, HFT)
auto-invokes:
- Leer, parsear o interactuar con pools de Raydium V2/V4
- Calcular swap_amount_out o predecir precios en Raydium
---

# Raydium V2 Integration (HFT Standard)

## Contexto de Invocación
Esta skill debe cargarse automáticamente al:
- Interactuar con contratos de Raydium (V2/V4/CPMM).
- Parsear cuentas de pools de liquidez de Raydium.
- Calcular `swap_amount_out` o simulaciones de precios sobre la curva Raydium.
- Analizar payloads y actualizaciones de cuentas provenientes de WebSockets o Geyser correspondientes a Raydium.

## 1. Reglas Cero-Tolerancia (Hot-Path)
1. **No RPC HTTP Polling:** Prohibido sugerir u operar usando llamadas `.getAccountInfo` a través de HTTP RPC para obtener el estado del pool durante el hot-path. El motor de ejecución se alimenta **exclusivamente** de los streams de baja latencia (Geyser).
2. **Zero-Allocation Parsing:** El casting de los bytes raw de la cuenta (`&[u8]`) devueltos por la RPC hacia el struct de la Pool de Raydium debe hacerse **exclusivamente** con `bytemuck::pod_read_unaligned` o casting `unsafe` estrictamente acotado.
   - **Prohibido:** Usar serialización dinámica como Borsh o Serde para deserializar estado del pool cuando estamos dentro del ciclo menor a 100µs.
3. **No Math Panics:** En cálculos de AMM (ej. math constante `x * y = k` o aritmética multivariable U256), usar siempre bloques de Safe Math como `.checked_add()`, `.checked_mul()`. Ningún cálculo matemático puede tener precondiciones implícitas sujetas a panics silenciosos. 

## 2. Estructura de Datos Estricta (bytemuck Pods)
Todos los estados recuperados de Raydium deben mapearse estáticamente mediante `#[repr(C)]` / `#[repr(packed)]` con los traits `Pod` y `Zeroable`.

```rust
use bytemuck::{Pod, Zeroable};

// Ejemplo obligatorio de simetría determinista
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct RaydiumAmmV4State {
    pub status: u64,
    pub nonce: u64,
    pub order_num: u64,
    pub depth: u64,
    pub coin_decimals: u64,
    pub pc_decimals: u64,
    pub state: u64,
    pub reset_flag: u64,
    pub min_size: u64,
    pub vol_max_cut_ratio: u64,
    pub amount_wave_ratio: u64,
    // La estructura debe cubrir los bytes exactos según el Anchor IDL.
}
```

## 3. Implementación Funcional (Error Handling)
1. Las funciones calculadoras de liquidez o precio de salida deben retornar *siempre* tuplas explícitas o structures ligeros envueltos en `Result<_, CriticalError>`.
2. **Never allocate memory:** Evitar `Box`, `String` o `Vec<T>` en las macros de swap interior de Raydium. Retornar los arrays on-stack.
