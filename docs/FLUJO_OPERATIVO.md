# 🚀 The Chassis - Flujo Operativo Completo

**Versión:** 1.0.0 - Auto-Buy + Auto-Sell  
**Fecha:** 2026-02-09  
**Estado:** Operativo con protección total

---

## 📋 Descripción General

The Chassis es ahora un motor completo de trading que gestiona **entrada** y **salida** de posiciones de forma semi-automatizada. El sistema prot tu capital con Stop-Loss automático y permite compras verificadas en segundos.

---

## 🔄 Flujo de Operación Estándar

### 1️⃣ **DETECCIÓN DE OPORTUNIDAD**
Encuentra un token nuevo en DexScreener, Telegram o Twitter.

### 2️⃣ **AUDITORÍA INSTANTÁNEA**
```bash
cd /home/ruben/Automatitation/bot_trading/intelligence/scripts
python3 auto_audit.py <CONTRACT_ADDRESS>
```

**Salida:**
- 🟢 **APROBADO**: Procede con compra
- 🟡 **RIESGO MEDIO**: Revisa manualmente
- 🔴 **PELIGRO**: Descarta

**El reporte se guarda automáticamente en:** `operational/audits/audit_<SYMBOL>_<TIMESTAMP>.md`

### 3️⃣ **COMPRA + REGISTRO AUTOMÁTICO**
```bash
cd /home/ruben/Automatitation/bot_trading/intelligence/scripts
python3 chassis_buy.py <SYMBOL> <MINT> <SOL_AMOUNT>
```

**Qué hace:**
1. Te muestra el link directo de Jupiter para comprar
2. Registra la compra en `targets.json` automáticamente
3. Configura:
   - Stop-Loss: -35%
   - Trailing Stop: Activo (+30% dispara, mantiene -20%)
   - Estado: Activo para monitoreo

**Ejemplo:**
```bash
python3 chassis_buy.py GENTLEMEN 5TATk16oMrt4vsMR8WwQ9AtiPeosdJhXFkp2UhGJpump 0.05
```

### 4️⃣ **ACTIVAR PROTECCIÓN**
```bash
cd /home/ruben/Automatitation/bot_trading/core/the_chassis
cargo run
```

**El bot ahora:**
- ✅ Monitorea el precio cada 5 segundos
- ✅ Calcula tú Drawdown en tiempo real
- ✅ Ejecuta venta automática si toca el -35%
- ✅ Te notifica por Telegram cada cambio importante
- ✅ Ajusta el Stop-Loss si el precio sube (Trailing)

---

## ⚙️ Configuración del Sistema

### `targets.json` (Gestión multi-token)
```json
{
  "targets": [
    {
      "symbol": "GENTLEMEN",
      "mint": "5TATk16oMrt4vsMR8WwQ9AtiPeosdJhXFkp2UhGJpump",
      "entry_price": 0.0003867,
      "amount_sol": 0.05,
      "stop_loss_percent": -35.0,
      "panic_sell_price": 0.0001,
      "active": true,
      "trailing_enabled": true,
      "trailing_distance_percent": 20.0,
      "trailing_activation_threshold": 30.0
    }
  ],
  "global_settings": {
    "min_sol_balance": 0.01,
    "jito_tip_lamports": 50000,
    "auto_execute": true,
    "monitor_interval_sec": 5
  }
}
```

### `.env` (Credenciales sensibles - NO comitear)
```bash
HELIUS_API_KEY=tu_api_key_aqui
WALLET_ADDRESS=tu_direccion_publica
WALLET_PRIVATE_KEY=tu_clave_privada_base58
TELEGRAM_BOT_TOKEN=tu_bot_token
TELEGRAM_CHAT_ID=tu_chat_id
```

---

## 🎯 Comandos Rápidos

### Auditar Token
```bash
cd intelligence/scripts
python3 auto_audit.py <MINT>
```

### Comprar Token
```bash
cd intelligence/scripts
python3 chassis_buy.py <SYMBOL> <MINT> <AMOUNT_SOL>
```

### Activar Monitor (Protección)
```bash
cd core/the_chassis
cargo run
```

### Ver Logs de Simulación
```bash
cat operational/logs/simulated_trades.csv
```

---

## 🛡️ System de Seguridad

1. **Auditoría Previa Obligatoria**: Nunca compres sin pasar por `auto_audit.py`
2. **Trailing Stop-Loss**: El SL sube contigo, asegurando ganancias
3. **Balance Mínimo**: El bot se apaga si tienes menos de 0.01 SOL
4. **Telegram Alerts**: Recibes notificación de TODO
5. **Logs Permanentes**: Todas las operaciones quedan registradas

---

## 📊 Indicadores de Salud

| Indicador | Estado | Descripción |
|-----------|--------|-------------|
| 🟢 Auto-Execute | ON | Vende automáticamente al tocar SL |
| 📱 Telegram | ACTIVO | Notificaciones en tiempo real |
| 🔑 Keypair | CARGADO | Listo para ejecutar transacciones |
| 💰 Balance | 0.1484 SOL | Suficiente para operar |

---

## 🚨 Qué hacer si...

### El bot no compra automáticamente
**Solución:** Por diseño, la compra es semi-manual (tú decides en Jup.ag). El bot se encarga de la **venta** automática.

### El precio cae pero no vende
1. Verifica que `auto_execute: true` en `targets.json`
2. Revisa que `WALLET_PRIVATE_KEY` esté en `.env`
3. Comprueba logs del bot

### Quiero cambiar el Stop-Loss
1. Edita `targets.json` (campo `stop_loss_percent`)
2. Reinicia el bot con `cargo run`

### Quiero vender manualmente AHORA
```bash
# Opción 1: Jup.ag manual
https://jup.ag/swap/<TOKEN_MINT>-SOL

# Opción 2: Parar bot y editar targets.json (poner active: false)
```

---

## 📈 Próximas Mejoras (Roadmap)

- [ ] **Compra automática vía Rust** (eliminar el paso manual)
- [ ] **Comando Telegram /buy**: Comprar desde el móvil
- [ ] **Backtesting con datos históricos**
- [ ] **Sniper Mode**: Compra en el bloque 0 de pools nuevas
- [ ] **gRPC Server**: Python ↔ Rust comunicación ultrarrápida

---

**Mantenido por:** Ruben  
**Licencia:** Privado  
**Soporte:** Este archivo 😎
