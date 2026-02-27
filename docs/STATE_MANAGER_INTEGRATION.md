# 🔄 State Manager — Estado Real v2.1

**Actualizado:** 2026-02-26  
**Estado:** ✅ COMPLETAMENTE INTEGRADO

> Este documento refleja el estado **actual e implementado** del sistema.
> La integración descrita aquí está en producción desde v2.1.

---

## ✅ Estado de Integración

| Componente | Estado | Notas |
|---|---|---|
| StateManager init en lib.rs | ✅ Activo | `trading_state.db` |
| Migración trades existentes | ✅ Activo | `fee_sol DEFAULT 0.0` |
| record_trade en BUY manual | ✅ v2.1 | `fee_sol` real |
| record_trade en PANIC/SELL | ✅ v2.1 | `fee_sol` real |
| record_trade en AUTO_TP1 | ✅ v2.1 | `fee_sol` real |
| record_trade en AUTO_TP2 | ✅ v2.1 | `fee_sol` real |
| record_trade en AUTO_SL | ✅ v2.1 | `fee_sol` real |
| FeeStats (`/fees` command) | ✅ v2.1 | Net PnL calculado |
| Posiciones persistentes | ✅ Activo | Inmunes a reinicios |

---

## Esquema SQLite Actual

### Tabla `trades`
```sql
CREATE TABLE IF NOT EXISTS trades (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    signature       TEXT    NOT NULL UNIQUE,
    token_mint      TEXT    NOT NULL,
    symbol          TEXT    NOT NULL,
    trade_type      TEXT    NOT NULL,  -- ver tipos abajo
    amount_sol      REAL    NOT NULL,
    tokens_amount   REAL    NOT NULL DEFAULT 0.0,
    price           REAL    NOT NULL DEFAULT 0.0,
    pnl_sol         REAL,              -- NULL si no calculado
    pnl_percent     REAL,              -- NULL si no calculado
    route           TEXT    NOT NULL DEFAULT '',
    price_impact_pct REAL   NOT NULL DEFAULT 0.0,
    fee_sol         REAL    NOT NULL DEFAULT 0.0,  -- NUEVO v2.1
    timestamp       INTEGER NOT NULL
);
```

### Tipos de trade registrados (`trade_type`)

| Valor | Origen | Cuándo |
|---|---|---|
| `MANUAL_BUY` | `/buy`, `/rbuy` | Compra manual via Telegram |
| `MANUAL_SELL` | `/panic`, `/panic_all` | Venta manual via Telegram |
| `AUTO_TP1` | Loop automático | Take Profit 1 alcanzado |
| `AUTO_TP2` | Loop automático | Take Profit 2 alcanzado |
| `AUTO_SL` | Loop automático | Stop-Loss de emergencia |
| `GHOST_PURGE` | Housekeeping | Posición cerrada sin TX real |

### Tabla `positions`
```sql
CREATE TABLE IF NOT EXISTS positions (
    id                           INTEGER PRIMARY KEY AUTOINCREMENT,
    token_mint                   TEXT    NOT NULL UNIQUE,
    symbol                       TEXT    NOT NULL,
    entry_price                  REAL    NOT NULL,
    amount_sol                   REAL    NOT NULL,
    current_price                REAL    NOT NULL,
    stop_loss_percent            REAL    NOT NULL,
    trailing_enabled             INTEGER NOT NULL DEFAULT 0,
    trailing_distance_percent    REAL    NOT NULL DEFAULT 5.0,
    trailing_activation_threshold REAL   NOT NULL DEFAULT 10.0,
    trailing_highest_price       REAL,
    trailing_current_sl          REAL,
    tp_percent                   REAL,
    tp_amount_percent            REAL,
    tp_triggered                 INTEGER NOT NULL DEFAULT 0,
    tp2_percent                  REAL,
    tp2_amount_percent           REAL,
    tp2_triggered                INTEGER NOT NULL DEFAULT 0,
    active                       INTEGER NOT NULL DEFAULT 1,
    created_at                   INTEGER NOT NULL,
    updated_at                   INTEGER NOT NULL
);
```

---

## API Pública del StateManager

### Posiciones
```rust
// Crear / actualizar posición
state_manager.upsert_position(&position).await?;

// Leer posición activa
state_manager.get_position(&token_mint).await?;

// Todas las posiciones activas
state_manager.get_active_positions().await?;

// Cerrar posición (active = false)
state_manager.close_position(&token_mint).await?;

// Actualizar precio en tiempo real
state_manager.update_position_price(&token_mint, price).await?;

// Marcar TP alcanzado
state_manager.mark_tp_triggered(&token_mint).await?;
state_manager.mark_tp2_triggered(&token_mint).await?;
```

### Trades
```rust
// Registrar un trade (con fee_sol real)
let trade = TradeRecord {
    id: None,
    signature: res.signature.clone(),
    token_mint: mint.clone(),
    symbol: symbol.clone(),
    trade_type: "AUTO_TP1".to_string(),
    amount_sol: res.output_amount,
    tokens_amount: res.input_amount,
    price: price_per_token,
    pnl_sol: Some(pnl),
    pnl_percent: Some(pnl_pct),
    route: res.route.clone(),
    price_impact_pct: res.price_impact_pct,
    fee_sol: res.fee_sol,  // real, del resultado del executor
    timestamp: chrono::Utc::now().timestamp(),
};
state_manager.record_trade(trade).await?;

// Historial
state_manager.get_trade_history(10).await?;

// Estadísticas de fees (para /fees command)
state_manager.get_fee_stats(Some(since_ts)).await?; // 24h
state_manager.get_fee_stats(None).await?;           // all-time
```

### FeeStats
```rust
pub struct FeeStats {
    pub total_fee_sol: f64,   // Suma de todos los fees
    pub total_trades: i64,    // Número de trades
    pub avg_fee_sol: f64,     // Promedio por trade
    pub gross_pnl_sol: f64,   // PnL bruto (sin fees)
    pub net_pnl_sol: f64,     // PnL neto = gross - total_fees
}
```

---

## Migración Automática

Al iniciar el bot, `StateManager::new()` ejecuta:
```sql
-- Añade columna fee_sol a bases de datos antiguas (idempotente)
ALTER TABLE trades ADD COLUMN fee_sol REAL NOT NULL DEFAULT 0.0;
```
Si la columna ya existe, el error se ignora silenciosamente.

---

## Archivos Relevantes

| Archivo | Responsabilidad |
|---|---|
| `src/state_manager.rs` | StateManager, TradeRecord, FeeStats, queries SQLite |
| `src/lib.rs` | Loop de monitoreo: TP1, TP2, SL → record_trade() |
| `src/telegram_commands.rs` | Comandos: /buy, /panic → record_trade() + /fees, /history |
| `src/executor_v2.rs` | Captura fee_sol real en SwapResult/BuyResult |
| `src/jupiter.rs` | SwapResult y BuyResult con campo fee_sol |
| `trading_state.db` | Base de datos SQLite (no subir al repositorio) |

---

## Notas de Producción

- **Backup:** `trading_state.db` contiene todo el historial de trades y posiciones. Hacer backup periódico.
- **Reset:** Para resetear el historial, borrar `trading_state.db` (se recrea automáticamente).
- **Concurrencia:** `deadpool-sqlite` gestiona el pool de conexiones. Seguro para operaciones concurrentes.
- **Resiliencia:** Todos los `record_trade()` y `close_position()` manejan errores explícitamente. Si la DB falla, se notifica vía Telegram y se registra en logs. **Nunca se ignoran errores de DB silenciosamente.**
