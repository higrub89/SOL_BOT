# 🤖 GUÍA RÁPIDA DE COMANDOS DE TELEGRAM (THE CHASSIS v2.0)

Esta es tu chuleta de batalla. Mantén este archivo abierto cuando operes.

---

## 🟢 ESTADO Y SALUD (Monitoring)

### `/ping`
**Uso:** Verificar si el bot está vivo y conectado a Solana.
**Respuesta:**
- `🏓 PONG - Health Check`
- RPC Latency (Verde < 200ms / Rojo > 500ms)
- Wallet Balance (Verde > 0.1 SOL / Rojo < 0.05 SOL)
- Estado (OPERATIVO / HIBERNANDO)

### `/balance`
**Uso:** Consultar cuánto SOL tienes disponible para operar.
**Respuesta:**
- `💰 BALANCE DE WALLET`
- `SOL: 1.2500`
- `USD (aprox): $185.50`

### `/status` (Legacy)
**Uso:** Ver el estado actual de todas las posiciones simuladas/reales en memoria.
**Respuesta:**
- `📊 STATUS DE POSICIONES`
- Lista de tokens con precio de entrada, actual, y drawdown %.
- Emojis: 🟢 (Ganando), 🟡 (Pérdida pequeña), 🔴 (Pérdida grande).

---

## 💰 OPERATIVA (Trading)

### `/buy <MINT> <CANTIDAD>`
**Uso:** Comprar un token INMEDIATAMENTE al precio actual de mercado.
**Ejemplo:** `/buy EKpQGSJpwMdD2vj7vj7t3H73h 0.1` (Compra 0.1 SOL de WIF)
**Respuesta:**
- `🚀 Iniciando Compra...`
- `✅ COMPRA EXITOSA` (con enlace a Solscan).
- O `❌ Error: Saldo insuficiente`.

### `/panic <MINT>`
**Uso:** 🚨 **BOTÓN DEL PÁNICO**. Vende el 100% de la posición inmediatamente.
**Ejemplo:** `/panic EKpQGSJpwMdD2vj7vj7t3H73h`
**Respuesta:**
- `🚨 PANIC SELL ACTIVADO`
- `✅ VENTA COMPLETADA` (recuperas SOL).

---

## 💾 PERSISTENCIA (Base de Datos)

### `/positions`
**Uso:** Ver las posiciones activas guardadas en la base de datos (inmunes a reinicios).
**Respuesta:**
- Lista similar a `/status`, pero confirmada por la DB.
- Muestra PnL acumulado real.

### `/history`
**Uso:** Ver los últimos 10 trades realizados.
**Respuesta:**
- `📜 HISTORIAL DE TRADES`
- Cada trade muestra: Hora, Tipo (BUY/SELL), Precio, PnL y Hash de transacción.

### `/stats`
**Uso:** Ver métricas globales de rendimiento.
**Respuesta:**
- `📈 ESTADÍSTICAS COMPLETAS`
- PnL Total acumulado (SOL).
- Win Rate (si implementado en futuro).
- Total de trades.

---

## 🛡️ SEGURIDAD Y CONTROL

### `/hibernate`
**Uso:** Detener TODA operación de trading automática. El bot sigue monitoreando pero NO ejecuta compras ni ventas.
**Respuesta:**
- `🛑 MODO HIBERNACIÓN ACTIVADO`

### `/wake`
**Uso:** Reactivar el trading automático.
**Respuesta:**
- `🟢 HIBERNACIÓN DESACTIVADA`

### `/targets`
**Uso:** Ver qué tokens está monitoreando el bot actualmente (desde `targets.json`).
**Respuesta:**
- Lista de `symbol` y `mint` con su Stop Loss configurado.
- Indica si `Auto-Execute` está ON/OFF globalmente.
