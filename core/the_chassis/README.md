# 🏎️ The Chassis - Solana Trading Engine

**v1.0.0** - Bot Interactivo con Trailing Stop-Loss y Monitor de Liquidez

## 🎯 ¿Qué es The Chassis?

The Chassis es un motor de trading automatizado e **interactivo** para Solana que monitorea tus posiciones en tiempo real, ajusta automáticamente tus stop-loss para proteger ganancias, detecta movimientos sospechosos de liquidez, y te permite controlarlo todo desde tu móvil con Telegram.

### ✨ Características Principales

- 🛡️ **Stop-Loss Dinámico**: Configura límites de pérdida personalizados por token
- 🎯 **Trailing Stop-Loss**: SL inteligente que sube automáticamente para proteger ganancias
- 🐋 **Monitor de Liquidez**: Detecta caídas de liquidez, spikes de volumen y posibles rug pulls
- 📱 **Bot Interactivo de Telegram**: Controla todo desde tu móvil con comandos en tiempo real
- 📊 **Multi-Target**: Monitorea múltiples tokens simultáneamente
- ⚡ **Ejecución Asistida**: Abre Jupiter automáticamente con el swap precargado
- 🔄 **Configuración en Caliente**: Cambia stop-loss sin recompilar
- 🎨 **Dashboard en Consola**: Visualización clara del estado de tus posiciones

## 🚀 Inicio Rápido

### 1. Requisitos Previos

```bash
# Rust (si no lo tienes instalado)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Git
sudo apt install git
```

### 2. Instalación

```bash
# Clonar el repositorio (si aún no lo tienes)
cd /home/ruben/Automatitation/bot_trading/core/the_chassis

# Compilar el proyecto
cargo build --release
```

### 3. Configuración Básica

#### a) Configurar `.env`

El archivo `.env` contiene tus credenciales:

```bash
HELIUS_API_KEY=tu_api_key_aqui
WALLET_ADDRESS=tu_wallet_aqui
MAX_LATENCY_MS=150

# Telegram (REQUERIDO para comandos interactivos)
TELEGRAM_BOT_TOKEN=tu_bot_token_aqui
TELEGRAM_CHAT_ID=tu_chat_id_aqui
```

#### b) Configurar `targets.json`

Edita `targets.json` para añadir los tokens que quieres monitorear:

```json
{
  "targets": [
    {
      "symbol": "MYTOKEN",
      "mint": "TokenMintAddressHere...",
      "entry_price": 0.001,
      "amount_sol": 0.1,
      "stop_loss_percent": -50.0,
      "panic_sell_price": 0.0005,
      "active": true,
      
      // Trailing Stop-Loss (opcional pero recomendado)
      "trailing_enabled": true,
      "trailing_distance_percent": 30.0,
      "trailing_activation_threshold": 50.0
    }
  ],
  "global_settings": {
    "min_sol_balance": 0.01,
    "jito_tip_lamports": 50000,
    "auto_execute": false,
    "monitor_interval_sec": 5
  }
}
```

### 4. Ejecutar el Bot

**Opción A: Script de inicio (recomendado)**
```bash
./start.sh
```

**Opción B: Directamente con cargo**
```bash
# Modo release (ejecución optimizada)
cargo run --release
```

## 📱 Bot Interactivo de Telegram

### Comandos Disponibles

Una vez que el bot está corriendo, puedes controlarlo desde Telegram:

- **`/start`** - Activa el bot y muestra la lista de comandos
- **`/status`** - Ver estado de TODOS tus tokens (precio, drawdown, valor actual)
- **`/balance`** - Consultar tu balance de SOL en la wallet
- **`/targets`** - Lista completa de tokens que estás monitoreando
- **`/help`** - Ayuda de todos los comandos

### Configurar Telegram (Obligatorio)

1. Habla con **@BotFather** en Telegram y crea un nuevo bot
2. Copia el token que te da
3. Habla con **@getidsbot** para obtener tu Chat ID
4. Añade ambos valores al archivo `.env`

👉 Lee la guía completa en **[TELEGRAM_SETUP.md](TELEGRAM_SETUP.md)**

## 🎯 Trailing Stop-Loss Inteligente

### ¿Qué es?

Un stop-loss que **sube automáticamente** cuando el precio sube, protegiendo tus ganancias.

### Ejemplo Práctico:

Imagina que compras un token a **$0.001** con SL al **-50%** (venta en $0.0005):

1. **Precio sube a $0.0015** (+50%) → Trailing SL se **ACTIVA**
2. **Nuevo SL dinámico**: En lugar de $0.0005, ahora es ~$0.00105
3. **Precio sigue a $0.002** → SL sube a ~$0.0014
4. **Precio cae a $0.0013** → **¡VENTA AUTOMÁTICA!**
   - Resultado: En lugar de perder -50%, ganas **+30%** 🎉

### Configuración:

```json
{
  "trailing_enabled": true,                    // Activar trailing SL
  "trailing_distance_percent": 30.0,           // Permite caer 30% desde el pico
  "trailing_activation_threshold": 50.0        // Se activa cuando ganas +50%
}
```

#### Parámetros Explicados:

- **`trailing_enabled`**: `true` para activar, `false` para usar SL fijo
- **`trailing_distance_percent`**: Cuánto puede caer desde el pico antes de vender
  - 20% = conservador (protege ganancias rápido)
  - 50% = agresivo (deja espacio para volatilidad)
- **`trailing_activation_threshold`**: A partir de qué ganancia se activa
  - 30% = se activa rápido
  - 100% = solo cuando duplicas tu inversión

## 🐋 Monitor de Liquidez y Detector de Ballenas

### ¿Qué Detecta?

El bot monitorea constantemente:
1. **Caídas dramáticas de liquidez** (posible rug pull)
2. **Spikes sospechosos de volumen** (ballenas entrando/saliendo)
3. **Señales de Rug Pull** (caída de precio + caída de liquidez simultánea)

### Alertas que Recibirás en Telegram:

#### 1. Alerta de Liquidez:
```
⚠️ ALERTA DE LIQUIDEZ - MYTOKEN

💧 Caída de liquidez: -35.2%
└─ Antes: $150,000
└─ Ahora: $97,000

🔍 Esto puede indicar ventas grandes o retiro de LP.
```

#### 2. Volumen Anormal:
```
📊 VOLUMEN ANORMAL - MYTOKEN

🚨 Spike de volumen: 8.5x del promedio
└─ Actual 24h: $850,000
└─ Promedio: $100,000

⚠️ Puede indicar actividad de ballenas o dump inminente.
```

#### 3. Advertencia de Rug Pull:
```
🚨🚨 ADVERTENCIA DE RUG PULL - MYTOKEN 🚨🚨

❌ Precio: -42.1%
❌ Liquidez: -58.3%

⚡ ACCIÓN INMEDIATA RECOMENDADA
Considera salir de la posición ahora.
```

## ⚙️ Configuración Avanzada

### Parámetros de `targets.json`

#### Por Token:
- **`symbol`**: Nombre del token (para visualización)
- **`mint`**: Dirección del token contract
- **`entry_price`**: Precio al que compraste
- **`amount_sol`**: Cantidad invertida en SOL
- **`stop_loss_percent`**: Límite de pérdida (ej: -50 = 50% de pérdida)
- **`panic_sell_price`**: Precio de pánico (opcional)
- **`active`**: true/false para activar/desactivar el monitoreo
- **`trailing_enabled`**: Activar trailing stop-loss
- **`trailing_distance_percent`**: Distancia del trailing desde el pico
- **`trailing_activation_threshold`**: Ganancia mínima para activar trailing

#### Configuración Global:
- **`min_sol_balance`**: Balance mínimo de SOL para operar
- **`jito_tip_lamports`**: Tip para Jito (si usas bundles)
- **`auto_execute`**: true = abre Jupiter automáticamente, false = solo alerta
- **`monitor_interval_sec`**: Intervalo de monitoreo en segundos

### Ejemplo Multi-Token con Trailing SL

```json
{
  "targets": [
    {
      "symbol": "SCALP_TOKEN",
      "mint": "...",
      "entry_price": 0.001,
      "amount_sol": 0.1,
      "stop_loss_percent": -30.0,
      "active": true,
      "trailing_enabled": true,
      "trailing_distance_percent": 20.0,    // Conservador
      "trailing_activation_threshold": 30.0  // Activa rápido
    },
    {
      "symbol": "HODL_TOKEN",
      "mint": "...",
      "entry_price": 0.0005,
      "amount_sol": 0.2,
      "stop_loss_percent": -50.0,
      "active": true,
      "trailing_enabled": true,
      "trailing_distance_percent": 50.0,    // Agresivo
      "trailing_activation_threshold": 100.0 // Solo si 2x
    }
  ],
  "global_settings": {
    "auto_execute": false,
    "monitor_interval_sec": 5
  }
}
```

## 📊 Ejemplo de Salida

```
╔════════════════════════════════════════════════════════════╗
║         🏎️  THE CHASSIS - Solana Trading Engine          ║
║       v1.0.0 - Interactive Bot + Trailing SL              ║
╚════════════════════════════════════════════════════════════╝

📂 Cargando configuración dinámica desde targets.json...
✅ Configuración cargada:
   • Targets activos: 2
   • Auto-Execute:    DESACTIVADO 🟡
   • Intervalo:       5s

🏦 WALLET STATUS:
   • Balance:   0.3124 SOL

───────────────────────────────────────────────────────────

⚡ EXECUTOR STATUS: SIMPLE (Browser-based)
📱 Telegram Command Handler: ACTIVADO

───────────────────────────────────────────────────────────

🛡️  EMERGENCY SYSTEM (Multi-Target):
   • Cargado: TOKEN1 (SL: -30%)
   • Cargado: TOKEN2 (SL: -50%)

═══════════════════════════════════════════════════════════
  🚀 INICIANDO MONITOR DINÁMICO v1.0.0
═══════════════════════════════════════════════════════════

┌────────────────────────────────────────────────────────┐
│ 🟢 TOKEN1 Status                                        │
├────────────────────────────────────────────────────────┤
│   Price:    $0.00125000                                │
│   Drawdown: +25.00%                                     │
│   SL Limit: -30.0% (Dist: 55.00%)                      │
│   🎯 Trailing SL: INACTIVE (activates at +30%)         │
└────────────────────────────────────────────────────────┘
```

## 🚨 ¿Qué Pasa Cuando se Activa el Stop-Loss?

### Si `auto_execute: false` (Modo Manual - Recomendado)
1. El bot detecta que el precio cayó por debajo del límite (o trailing SL)
2. Muestra una alerta grande en la consola
3. Envía notificación a Telegram con el link de Jupiter
4. **TÚ DECIDES** si ejecutar la venta o no

### Si `auto_execute: true` (Modo Automático)
1. El bot detecta el stop-loss
2. Genera la URL de Jupiter con el swap precargado
3. **Abre tu navegador automáticamente**
4. Envía notificación a Telegram
5. Confirmas la transacción en Jupiter manualmente

## 🔒 Seguridad

⚠️ **IMPORTANTE**: Este bot **NO tiene acceso a tu wallet**. Solo:
- Consulta precios públicos
- Lee el balance de tu wallet (solo lectura)
- Genera URLs de Jupiter
- Envía notificaciones a Telegram

**Siempre confirmas manualmente** las transacciones en Jupiter.

## 🐛 Solución de Problemas

### "Telegram Command Handler: DESACTIVADO"
- Verifica que `TELEGRAM_BOT_TOKEN` y `TELEGRAM_CHAT_ID` estén en `.env`
- Lee `TELEGRAM_SETUP.md` para la configuración completa

### "Error obteniendo precio de {TOKEN}"
- Verifica que el `mint` sea correcto
- El token podría no tener liquidez suficiente
- Problema temporal de red/API

### "WALLET_ADDRESS not found"
- Asegúrate de que el archivo `.env` está en el directorio correcto
- Verifica que no haya espacios extras en el `.env`

### El bot responde múltiples veces en Telegram
- Reinicia el bot con `cargo build --release && ./target/release/the_chassis`
- Esto actualizará el offset de mensajes de Telegram

### El navegador no se abre automáticamente
- Verifica que `auto_execute: true` en `targets.json`
- Prueba ejecutar manualmente: `xdg-open https://jup.ag`

## 📁 Estructura del Proyecto

```
the_chassis/
├── src/
│   ├── main.rs                # Punto de entrada principal
│   ├── config.rs              # Carga de targets.json
│   ├── scanner.rs             # Monitoreo de precios
│   ├── emergency.rs           # Lógica de stop-loss
│   ├── executor_simple.rs     # Generación de URLs Jupiter
│   ├── telegram.rs            # Notificaciones Telegram
│   ├── telegram_commands.rs   # 🆕 Bot interactivo
│   ├── trailing_sl.rs         # 🆕 Trailing stop-loss
│   ├── liquidity_monitor.rs   # 🆕 Monitor de liquidez
│   ├── jupiter.rs             # Integración Jupiter API
│   └── ...
├── targets.json               # ⚙️ TU CONFIGURACIÓN PRINCIPAL
├── .env                       # 🔑 Credenciales (NO COMPARTIR)
├── start.sh                   # 🚀 Script de inicio rápido
├── TELEGRAM_SETUP.md          # 📱 Guía de Telegram
├── ADVANCED_FEATURES.md       # 📖 Guía de features avanzadas
├── FEATURES_SUMMARY.md        # 📋 Resumen ejecutivo
└── README.md                  # 📖 Este archivo
```

## 🛣️ Roadmap

### ✅ Completado (v1.0.0)
- [x] Sistema de monitoreo multi-target
- [x] Stop-loss dinámico configurable
- [x] Integración con Jupiter API
- [x] Notificaciones Telegram
- [x] Ejecución asistida (browser-based)
- [x] **BOT de Telegram para comandos interactivos** ✨
- [x] **Trailing stop-loss inteligente** ✨
- [x] **Monitor de liquidez y detector de rug pulls** ✨

### 🚧 En Progreso
- [ ] Dashboard web en tiempo real
- [ ] Auto-ejecución con firma de transacciones
- [ ] Integración con Jito bundles
- [ ] Historial de trades y performance

### 🔮 Futuro
- [ ] Indicadores técnicos (RSI, MACD)
- [ ] Backtesting de estrategias
- [ ] Machine Learning para detección de patrones
- [ ] Soporte para múltiples blockchains

## 📝 Changelog

### v1.0.0 (2026-02-08) 🎉
- ✨ Bot interactivo de Telegram con comandos en tiempo real
- 🎯 Sistema de trailing stop-loss para proteger ganancias
- 🐋 Monitor de liquidez y detector de rug pulls
- 📱 Comandos: /status, /balance, /targets, /help
- 🛠️ Fix: Prevención de spam en notificaciones de Telegram
- 📖 Documentación completa de nuevas features

### v0.9.0 (2026-02-08)
- ✨ Añadidas notificaciones de Telegram
- ⚡ Mejorado el flujo de alertas de stop-loss
- 📱 Script de inicio rápido (`start.sh`)
- 📖 Documentación completa de setup

### v0.8.0
- 🔄 Configuración dinámica sin recompilación
- 📊 Soporte multi-target
- 🎨 Dashboard mejorado en consola

## 🤝 Contribuir

Este es un proyecto personal, pero si tienes sugerencias:
1. Abre un issue
2. Propón mejoras
3. Comparte tu experiencia

## ⚖️ Licencia

Uso personal. No redistribuir sin permiso.

---

**⚠️ DISCLAIMER**: Este bot es una herramienta de asistencia. El trading de criptomonedas implica riesgos significativos. Usa bajo tu propia responsabilidad.

---

Desarrollado con ⚡ por Ruben | 2026
