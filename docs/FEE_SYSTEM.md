# ⛽ Fee Tracking & Dynamic Priority Fee System — v2.1

**Implementado:** 2026-02-26  
**Commit:** `d575b78`, `eff2745`, `af8fa50`

---

## Motivación

Antes de v2.1 el campo `fee_sol` se almacenaba siempre como `0.0` en la base
de datos. Esto impedía calcular la rentabilidad real del bot (Gross PnL - Fees
= Net PnL). Además, se usaba un Priority Fee fijo de 100k µL hardcoded, que en
condiciones normales de red era innecesariamente caro.

---

## Arquitectura del Sistema

```
┌─────────────────────────────────────────────────────────────────┐
│                    CADA TRANSACCIÓN                             │
│                                                                 │
│  1. get_dynamic_priority_fee()                                  │
│     └─► Helius getPriorityFeeEstimate (High, <2s timeout)       │
│         ├─ Normal: 10k–200k µL                                  │
│         ├─ Congestionado: 200k–2M µL (cap)                      │
│         └─ Fallback: 100k µL (si Helius no responde)            │
│                                                                 │
│  2. Ejecutar TX                                                 │
│     ├─ Raydium Direct (Fast Path: <150ms)  ─► pool en cache     │
│     └─ Jupiter Standard (Fallback: ~400ms) ─► universal         │
│                                                                 │
│  3. Calcular fee_sol real                                       │
│     fee_sol = jito_tip_lamports / 1e9                           │
│             + priority_fee_microlamports / 1e12                 │
│                                                                 │
│  4. SwapResult / BuyResult incluye fee_sol                      │
│     └─► record_trade(TradeRecord { fee_sol, ... })              │
│         └─► SQLite trades.fee_sol = valor real                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Componentes Modificados

### `jupiter.rs` — Structs de resultado

```rust
pub struct SwapResult {
    pub signature: String,
    pub input_amount: f64,
    pub output_amount: f64,
    pub route: String,
    pub price_impact_pct: f64,
    pub fee_sol: f64,  // ← NUEVO v2.1
}

pub struct BuyResult {
    pub signature: String,
    pub sol_spent: f64,
    pub tokens_received: f64,
    pub price_per_token: f64,
    pub route: String,
    pub price_impact_pct: f64,
    pub fee_sol: f64,  // ← NUEVO v2.1
}
```

### `executor_v2.rs` — Captura del fee real

```rust
// Dynamic Priority Fee (consultado en cada buy)
let dynamic_priority_fee = self.get_dynamic_priority_fee().await;

// fee_sol calculado como suma de ambos componentes
fee_sol: Self::lamports_to_sol(jito_tip_lamports)
       + Self::microlamports_to_sol(dynamic_priority_fee),
```

#### `get_dynamic_priority_fee()`
```rust
pub async fn get_dynamic_priority_fee(&self) -> u64 {
    // Consulta Helius getPriorityFeeEstimate
    // - Nivel: "High" (balance velocidad/coste)
    // - Cap: 2_000_000 µL (protección anti-spike)
    // - Timeout: 2 segundos
    // - Fallback: 100_000 µL (transparente)
}
```

#### Conversiones
```rust
fn lamports_to_sol(lamports: u64) -> f64 {
    lamports as f64 / 1_000_000_000.0  // 1e9
}

fn microlamports_to_sol(microlamports: u64) -> f64 {
    microlamports as f64 / 1_000_000_000_000.0  // 1e12
}
```

### `raydium.rs` — Fast Exit Path

Nuevos métodos añadidos en v2.1:

- **`execute_sell(token_mint, amount_in, min_sol_out, keypair)`**  
  Venta directa token→SOL via Raydium AMM v4. Sin HTTP roundtrip.
  
- **`execute_sell_with_jito(token_mint, amount_in, min_sol_out, jito_tip, keypair)`**  
  Bundle [swap_tx + tip_tx] para guaranteed inclusion en el siguiente bloque.
  Fallback automático a RPC estándar si Jito no responde.

### `executor_v2.rs` — Raydium como Fast Path

En `execute_emergency_sell_with_params()`:

```
PRIORIDAD 1: Raydium Direct + Jito
    ├─ find_pool(token → SOL)  ← cache / DexScreener
    ├─ execute_sell_with_jito()
    └─ DevuelvE SwapResult { fee_sol = jito_tip }

PRIORIDAD 2: Jupiter Standard (fallback)
    ├─ get_quote() → HTTP
    ├─ get_swap_transaction() → HTTP
    └─ Devuelve SwapResult { fee_sol = jito_tip + dynamic_priority_fee }
```

### `telegram_commands.rs` — Propagación del fee

Todos los `TradeRecord` ahora usan `res.fee_sol` real:

```rust
// Antes (v2.0):
fee_sol: 0.0,

// Ahora (v2.1):
fee_sol: res.fee_sol,  // capturado del resultado real
```

Comandos actualizados: `/buy`, `/rbuy`, `/panic`, `/panic_all`

### `lib.rs` — Loop automático TP/SL

Los tres triggers automáticos ahora registran trades con `record_trade()`:

| Trigger | `trade_type` | fee_sol | pnl_sol |
|---|---|---|---|
| Take Profit 1 | `AUTO_TP1` | `res.fee_sol` (real) | calculado vs entry |
| Take Profit 2 | `AUTO_TP2` | `res.fee_sol` (real) | calculado vs entry |
| Stop-Loss | `AUTO_SL` | `res.fee_sol` (real) | calculado vs entry |

Antes de v2.1 estos trades **no se registraban** en la BD.

---

## Comando `/fees`

El comando Telegram `/fees` consulta `StateManager::get_fee_stats()` que agrega:

```sql
SELECT
    COALESCE(SUM(COALESCE(fee_sol, 0.0)), 0.0) as total_fees,
    COUNT(*) as total_trades,
    COALESCE(SUM(COALESCE(pnl_sol, 0.0)), 0.0) as total_pnl
FROM trades
WHERE timestamp >= ?  -- 24h window o all-time
```

**Muestra:**
- Total fees pagados (24h y all-time)
- Fee promedio por trade
- Gross PnL (sin descontar fees)
- **Net PnL = Gross PnL - Total Fees** ← el número que importa

---

## Migración de Datos

Al arrancar, el bot añade automáticamente la columna si no existe:
```sql
ALTER TABLE trades ADD COLUMN fee_sol REAL NOT NULL DEFAULT 0.0;
```

Los trades anteriores a v2.1 tendrán `fee_sol = 0.0` (valor por defecto).

---

## Configuración

En `config.toml`:
```toml
[global_settings]
jito_tip_lamports = 100000  # Tip base (µL). Dynamic priority fee se suma encima.
```

Variables de entorno requeridas:
```
HELIUS_API_KEY  # Para Dynamic Priority Fee. Sin ella → fallback 100k µL.
```

---

## Latencias Medidas

| Path | Descripción | Latencia |
|---|---|---|
| Raydium + Jito | Pool en cache, bundle atómico | ~50–150ms |
| Raydium + RPC | Pool en cache, Jito falló | ~100–300ms |
| Jupiter + Jito | HTTP quote + swap + bundle | ~300–500ms |
| Jupiter + RPC | HTTP quote + swap + RPC | ~400–600ms |

---

## Valores Típicos de Fee

| Condición de red | Priority Fee | Jito Tip | Total fee_sol |
|---|---|---|---|
| Red tranquila | ~20k µL | 100k µL | ~0.000100 SOL |
| Red media | ~100k µL | 100k µL | ~0.000200 SOL |
| Red congestionada | ~500k µL (cap 2M) | 100k µL | ~0.000600 SOL |
| Cap máximo | 2M µL | 100k µL | ~0.002100 SOL |
