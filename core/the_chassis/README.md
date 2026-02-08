# 🏎️ The Chassis - Solana Trading Engine

**v0.9.0** - Sistema de Stop-Loss Automático con Notificaciones Telegram

## 🎯 ¿Qué es The Chassis?

The Chassis es un motor de trading automatizado para Solana que monitorea tus posiciones en tiempo real y te alerta cuando se activa el stop-loss. Está diseñado para proteger tu capital en tokens de alto riesgo.

### ✨ Características Principales

- 🛡️ **Stop-Loss Dinámico**: Configura límites de pérdida personalizados por token
- 📱 **Alertas Telegram**: Notificaciones instantáneas en tu móvil cuando se activa el SL
- ⚡ **Ejecución Asistida**: Abre Jupiter automáticamente con el swap precargado
- 📊 **Multi-Target**: Monitorea múltiples tokens simultáneamente
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

El archivo `.env` ya está configurado con tus credenciales:

```bash
HELIUS_API_KEY=1d8b1813-084e-41ed-8e93-87a503c496c6
WALLET_ADDRESS=6EJeiMFoBgQrUfbpt8jjXZdc5nASe2Kc8qzfVSyGrPQv
MAX_LATENCY_MS=150

# Telegram (Opcional - sigue TELEGRAM_SETUP.md para configurar)
TELEGRAM_BOT_TOKEN=
TELEGRAM_CHAT_ID=
```

#### b) Configurar `targets.json`

Edita `targets.json` para añadir los tokens que quieres monitorear:

```json
{
  "targets": [
    {
      "symbol": "ICEBEAR",
      "mint": "86WM5NBUtRWTHULKrspS1TdzVFAcZ9buXsGRAiFDpump",
      "entry_price": 0.0005687,
      "amount_sol": 0.051,
      "stop_loss_percent": -50.0,
      "panic_sell_price": 0.00028,
      "active": true
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
# Modo debug (compilación rápida)
cargo run

# Modo release (ejecución optimizada)
cargo run --release
```

## 📱 Configurar Notificaciones Telegram (Recomendado)

Las notificaciones de Telegram te permiten recibir alertas instantáneas en tu móvil. Lee la guía completa en:

👉 **[TELEGRAM_SETUP.md](TELEGRAM_SETUP.md)**

Resumen rápido:
1. Crea un bot con @BotFather
2. Obtén tu Chat ID con @getidsbot
3. Añade las credenciales al archivo `.env`

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

#### Configuración Global:
- **`min_sol_balance`**: Balance mínimo de SOL para operar
- **`jito_tip_lamports`**: Tip para Jito (si usas bundles)
- **`auto_execute`**: true = abre Jupiter automáticamente, false = solo alerta
- **`monitor_interval_sec`**: Intervalo de monitoreo en segundos

### Ejemplo Multi-Token

```json
{
  "targets": [
    {
      "symbol": "TOKEN1",
      "mint": "...",
      "entry_price": 0.001,
      "amount_sol": 0.1,
      "stop_loss_percent": -30.0,
      "active": true
    },
    {
      "symbol": "TOKEN2",
      "mint": "...",
      "entry_price": 0.0005,
      "amount_sol": 0.05,
      "stop_loss_percent": -50.0,
      "active": true
    }
  ],
  "global_settings": {
    "auto_execute": true,
    "monitor_interval_sec": 3
  }
}
```

## 📊 Ejemplo de Salida

```
╔════════════════════════════════════════════════════════════╗
║         🏎️  THE CHASSIS - Solana Trading Engine          ║
║       v0.9.0 - Dynamic Config (Zero Recompile)            ║
╚════════════════════════════════════════════════════════════╝

📂 Cargando configuración dinámica desde targets.json...
✅ Configuración cargada:
   • Targets activos: 1
   • Auto-Execute:    DESACTIVADO 🟡
   • Intervalo:       5s

🏦 WALLET STATUS:
   • Balance:   0.0512 SOL

───────────────────────────────────────────────────────────

⚡ EXECUTOR STATUS: SIMPLE (Browser-based)
📱 Telegram Notifier: ACTIVADO
   • Chat ID: 123456789

───────────────────────────────────────────────────────────

🛡️  EMERGENCY SYSTEM (Multi-Target):
   • Cargado: ICEBEAR (SL: -50%)

═══════════════════════════════════════════════════════════
  🚀 INICIANDO MONITOR DINÁMICO v0.9.0
═══════════════════════════════════════════════════════════

┌────────────────────────────────────────────────────────┐
│ 🟢 ICEBEAR Status                                       │
├────────────────────────────────────────────────────────┤
│   Price:    $0.00045123                                │
│   Drawdown: -20.67%                                     │
│   SL Limit: -50.0% (Dist: 29.33%)                      │
└────────────────────────────────────────────────────────┘
```

## 🚨 ¿Qué Pasa Cuando se Activa el Stop-Loss?

### Si `auto_execute: false` (Modo Manual)
1. El bot detecta que el precio cayó por debajo del límite
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

**Siempre confirmas manualmente** las transacciones en Jupiter.

## 🐛 Solución de Problemas

### "Error obteniendo precio de {TOKEN}"
- Verifica que el `mint` sea correcto
- El token podría no tener liquidez suficiente
- Problema temporal de red/API

### "WALLET_ADDRESS not found"
- Asegúrate de que el archivo `.env` está en el directorio correcto
- Verifica que no haya espacios extras en el `.env`

### "Telegram Notifier: DESACTIVADO"
- Es normal si no has configurado Telegram
- Lee `TELEGRAM_SETUP.md` para activarlo

### El navegador no se abre automáticamente
- Verifica que `auto_execute: true` en `targets.json`
- Prueba ejecutar manualmente: `xdg-open https://jup.ag`

## 📁 Estructura del Proyecto

```
the_chassis/
├── src/
│   ├── main.rs              # Punto de entrada principal
│   ├── config.rs            # Carga de targets.json
│   ├── scanner.rs           # Monitoreo de precios
│   ├── emergency.rs         # Lógica de stop-loss
│   ├── executor_simple.rs   # Generación de URLs Jupiter
│   ├── telegram.rs          # Notificaciones Telegram
│   ├── jupiter.rs           # Integración Jupiter API
│   └── ...
├── targets.json             # ⚙️ TU CONFIGURACIÓN PRINCIPAL
├── .env                     # 🔑 Credenciales (NO COMPARTIR)
├── start.sh                 # 🚀 Script de inicio rápido
├── TELEGRAM_SETUP.md        # 📱 Guía de Telegram
└── README.md                # 📖 Este archivo
```

## 🛣️ Roadmap

### ✅ Completado
- [x] Sistema de monitoreo multi-target
- [x] Stop-loss dinámico configurable
- [x] Integración con Jupiter API
- [x] Notificaciones Telegram
- [x] Ejecución asistida (browser-based)

### 🚧 En Progreso
- [ ] Dashboard web en tiempo real
- [ ] Auto-ejecución con firma de transacciones
- [ ] Integración con Jito bundles
- [ ] Historial de trades y performance

### 🔮 Futuro
- [ ] BOT de Telegram para comandos interactivos
- [ ] Soporte para trailing stop-loss
- [ ] Indicadores técnicos (RSI, MACD)
- [ ] Backtesting de estrategias

## 📝 Changelog

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
