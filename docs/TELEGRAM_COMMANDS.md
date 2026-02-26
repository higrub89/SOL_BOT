# 🤖 GUÍA DE COMANDOS DE TELEGRAM — THE CHASSIS v2.1

> **Última actualización:** 2026-02-26  
> **Versión del bot:** v2.1 (Real Fee Tracking + Raydium Direct SELL + Dynamic Priority Fee)

Esta es tu chuleta de batalla. Mantén este archivo abierto cuando operes.

---

## 🟢 ESTADO Y SALUD (Monitoring)

### `/ping`
**Uso:** Verificar si el bot está vivo y conectado a Solana.  
**Respuesta:**
- `🏓 PONG - Health Check`
- RPC Latency (Verde <200ms / Rojo >500ms)
- Wallet Balance (Verde >0.1 SOL / Rojo <0.05 SOL)
- Estado (OPERATIVO / HIBERNANDO)

### `/balance`
**Uso:** Consultar cuánto SOL tienes disponible para operar.  
**Respuesta:**
```
💰 BALANCE DE WALLET
SOL: 1.2500
USD (aprox): $185.50
```

### `/status`
**Uso:** Ver el estado actual de todas las posiciones activas en memoria.  
**Respuesta:** Lista de tokens con precio de entrada, actual, drawdown % y PnL.  
Emojis: 🟢 (Ganando) · 🟡 (Pérdida pequeña) · 🔴 (Stop-Loss inminente)

---

## 💰 OPERATIVA (Trading)

### `/buy <MINT> <CANTIDAD>`
**Uso:** Comprar un token inmediatamente al precio actual de mercado.  
**Ejemplo:** `/buy EKpQGSJpwMdD2vj7vj7t3H73h 0.1`  
**Flujo interno:** Jupiter quote → Jito Bundle → Registra en DB con `fee_sol` real.  
**Respuesta:**
- `🚀 Iniciando Compra...`
- `✅ COMPRA EXITOSA` (con enlace Solscan + fee pagado)
- O `❌ Error: Saldo insuficiente`

### `/rbuy <MINT> <CANTIDAD>`
**Uso:** Comprar un token vía **Raydium Direct** (ultra-baja latencia, prioridad absoluta).  
Se usa cuando el pool ya está en cache local. Fallback automático a Jupiter si falla.  
**Ejemplo:** `/rbuy EKpQGSJpwMdD2vj7vj7t3H73h 0.1`

### `/panic <MINT>`
**Uso:** 🚨 **BOTÓN DEL PÁNICO**. Vende el 100% de la posición inmediatamente.  
**Flujo v2.1:** Intenta Raydium Direct primero (<150ms) → Fallback Jupiter (~400ms).  
**Ejemplo:** `/panic EKpQGSJpwMdD2vj7vj7t3H73h`  
**Respuesta:**
- `🚨 PANIC SELL ACTIVADO`
- `✅ VENTA COMPLETADA` (ruta usada + fee pagado)

### `/panic_all`
**Uso:** 🚨 Vende **TODOS** los tokens en posición simultáneamente via Jito Bundle.  
Un único bundle de transacciones para liquidar toda la cartera de una vez.

---

## 📊 FEES Y RENTABILIDAD (Nuevo v2.1)

### `/fees`
**Uso:** Ver estadísticas detalladas de fees pagados y PnL neto real.  
**Respuesta:**
```
⛽ FEE ANALYTICS

📅 Últimas 24h
  Trades:     12
  Total Fees: 0.002400 SOL
  Avg Fee:    0.000200 SOL/trade

📆 All-Time
  Trades:     87
  Total Fees: 0.017400 SOL
  Avg Fee:    0.000200 SOL/trade
  Gross PnL:  +0.4200 SOL
  Net PnL:    +0.4026 SOL  ← PnL real después de fees

ℹ️ fee_sol capturado desde v2.1 en adelante.
```
> **Nota:** Muestra el verdadero impacto de las tarifas en tu rentabilidad.
> Incluye todos los trades: manuales (/buy, /panic) y automáticos (TP1, TP2, SL).

---

## 💾 PERSISTENCIA (Base de Datos SQLite)

### `/positions`
**Uso:** Ver las posiciones activas guardadas en DB (inmunes a reinicios del bot).  
**Respuesta:** Lista con precio de entrada, actual, PnL acumulado y SL configurado.

### `/history`
**Uso:** Ver los últimos 10 trades realizados.  
**Respuesta:**
```
📜 HISTORIAL DE TRADES
BUY   TOKEN        0.0001200 SOL  · Fee: 0.0002 SOL
SELL  TOKEN  TP1   0.0001800 SOL  · PnL: +50%
SL    TOKEN        0.0000900 SOL  · PnL: -25%
```
Tipos registrados desde v2.1:
- `MANUAL_BUY` — Compra manual via `/buy` o `/rbuy`
- `MANUAL_SELL` — Venta manual via `/panic` o `/panic_all`
- `AUTO_TP1` — Take Profit 1 automático del bot
- `AUTO_TP2` — Take Profit 2 automático (moonbag)
- `AUTO_SL` — Stop-Loss de emergencia automático
- `GHOST_PURGE` — Posiciones cerradas sin transacción real

### `/stats`
**Uso:** Ver métricas globales de rendimiento.  
**Respuesta:**
```
📈 ESTADÍSTICAS COMPLETAS
PnL Total acumulado (SOL)
Total de trades realizados
```

---

## 🛡️ SEGURIDAD Y CONTROL

### `/hibernate`
**Uso:** Detener TODA operación de trading automática.  
El bot sigue monitoreando precios pero NO ejecuta compras ni ventas automáticas.  
Los SL en modo hibernación **envían alerta** para que ejecutes manualmente en Jupiter.  
**Respuesta:** `🛑 MODO HIBERNACIÓN ACTIVADO`

### `/wake`
**Uso:** Reactivar el trading automático.  
**Respuesta:** `🟢 HIBERNACIÓN DESACTIVADA · Auto-execute: ON`

### `/targets`
**Uso:** Ver qué tokens está monitoreando el bot (desde `targets.json`).  
**Respuesta:** Lista de `symbol`, `mint`, Stop Loss configurado, TP1/TP2 targets.  
Indica si `Auto-Execute` está ON/OFF globalmente.

---

## ⛽ SISTEMA DE FEES (v2.1)

### Cómo se calculan los fees

| Componente | Origen | Valor típico |
|---|---|---|
| **Jito Tip** | `config.toml → jito_tip_lamports` | 0.0001 SOL |
| **Priority Fee** | Helius API `getPriorityFeeEstimate` (High) | Variable (10k–500k µL) |
| **Total fee_sol** | Jito + Priority (en SOL) | ~0.0001–0.0015 SOL |

### Dynamic Priority Fee (v2.1)
El bot consulta Helius en cada transacción de compra para obtener el fee óptimo:
- **Congestionado:** Sube automáticamente (hasta 2M µL máximo)
- **Tranquilo:** Baja automáticamente (ahorro real vs fee fijo)
- **Helius down:** Fallback a 100k µL (transparente)

### Rutas de ejecución y latencia

| Escenario | Ruta | Latencia estimada |
|---|---|---|
| Compra estándar | Jupiter + Jito | ~300-500ms |
| Compra Raydium | Raydium Direct + Jito | ~50-150ms |
| Venta emergencia (pool en cache) | **Raydium Direct + Jito** | **~50-150ms** |
| Venta emergencia (pool nuevo) | Jupiter + Jito (fallback) | ~300-500ms |
| Venta pánico bundle | Jupiter Multi-Sell + Jito | ~400-600ms |

---

## 🔄 FLUJO AUTOMÁTICO (Sin intervención manual)

El bot ejecuta automáticamente cuando `auto_execute: true` en `config.toml`:

```
Precio actualizado (cada ~5s vía WS)
        ↓
¿Gana >= TP1 target?  → AUTO_TP1 (vende X% → registra en DB con fee real)
¿Gana >= TP2 target?  → AUTO_TP2 (vende resto → registra en DB con fee real)
¿Cae <= SL límite?    → AUTO_SL  (vende 100% → registra en DB con fee real)
        ↓
Telegram notifica con fee pagado incluido en el mensaje
```

**Nota:** Los trades automáticos ahora aparecen en `/history` y `/fees` desde v2.1.

---

## 📋 REFERENCIA RÁPIDA

```
/ping          → Estado del bot
/balance       → SOL disponible
/status        → Posiciones en memoria
/positions     → Posiciones en DB (persistentes)
/history       → Últimos 10 trades
/fees          → Analytics de fees y PnL neto ← NUEVO v2.1
/stats         → Métricas generales
/buy M C       → Comprar token M con C SOL
/rbuy M C      → Comprar vía Raydium Direct
/panic M       → Vender 100% del token M (Fast Exit)
/panic_all     → Vender TODOS los tokens
/hibernate     → Pausar trading automático
/wake          → Reanudar trading automático
/targets       → Ver tokens monitoreados
/help          → Esta guía
```
