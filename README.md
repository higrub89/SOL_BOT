# 🚀 SOL_BOT - Motor de Trading de "Ultralujo y Sistemas Críticos"

**Autor:** Rubén  
**Entorno:** Ubuntu/Linux  
**Filosofía:** Ingeniería de precisión (Estilo MV Agusta / Ferrari / 42 Madrid)  
**Stack Principal:** Rust (Chassis Engine), Python (Intelligence), Jupiter/Helius (Infrastructure)

---

## 🏎️ El Concepto: "The Chassis" v1.1.0

Este no es un bot genérico. Es un **chasis de alto rendimiento** diseñado para competir en el Gran Premio de las memecoins de Solana. Siguiendo el rigor técnico de **42 Madrid**, hemos pasado de un sistema de monitoreo pasivo a un **ecosistema de ejecución reactiva de baja latencia**.

## 🏗️ Arquitectura Mejorada (Hiperluxury Standard)

```
bot_trading/
├── core/                # 🚀 MOTOR DE EJECUCIÓN (The Chassis)
│   ├── src/             
│   │   ├── main.rs      # Orquestador con soporte CLI (Buy/Scan/Monitor)
│   │   ├── executor_v2.rs # Motor de Swaps (Jupiter v6 Integration)
│   │   ├── websocket.rs # Sensor de Telemetría (Logs Listener) + Auto-reconnect
│   │   ├── raydium.rs   # 🆕 Direct Swap Engine (Bypass Jupiter) - EN DESARROLLO
│   │   └── telegram_commands.rs # Control remoto + /buy command
│   └── proto/           # Contratos gRPC para Fase 3
│
├── intelligence/        # 🧠 MÓDULO DE INTELIGENCIA (Auto-Audit)
│   └── scripts/         
│       ├── auto_audit.py  # Auditoría 2s (RugCheck + DexScreener API)
│       └── chassis_buy.py # Orquestador de compra semi-automática (Python Fallback)
│
├── operational/         # 📊 DEPÓSITO DE DATOS & LOGS
│   ├── audits/          # Reportes históricos (🟢/🟡/🔴)
│   └── logs/            # Registros de Paper Trading y Ejecución Real
│
├── docs/                # 📚 ROADMAP & ESPECIFICACIONES TÉCNICAS
├── DEPLOYMENT.md        # 🆕 Guía completa para hosting en servidor VPS
└── start_bot.sh         # 🆕 Script de arranque con menú interactivo
```

---

## 🛠️ Capacidades Actuales de Competición

### 1. 🧠 Módulo de Inteligencia (Auto-Audit)
Hemos eliminado el cuello de botella de la auditoría manual.
- **Velocidad:** 2 segundos por token.
- **Rigor:** Consulta directa a los "Storage" de Solana para verificar autoridades (Mint/Freeze) y liquidez bloqueada.
- **Uso:** `python3 intelligence/scripts/auto_audit.py <MINT_ADDRESS>`

### 2. 💰 Sistema de Ejecución Directa (CLI + Telegram)
Ya no dependemos de dashboards lentos. El bot tiene "dedos" propios.
- **CLI:** `cargo run -- buy --mint <MINT> --sol <CANTIDAD>`
- **Telegram:** `/buy <MINT> <SOL>` desde tu móvil
- **Ventaja:** Swaps directos vía Jupiter Aggregator con cálculo de slippage dinámico.
- **Estado:** ⚠️ Requiere conexión estable a `quote-api.jup.ag` (ver Roadmap Raydium)

### 📡 3. Sensor de Telemetría (WebSocket logs) + Auto-Reconnect
"Escuchamos" la red, no preguntamos por ella.
- **Tipo:** `logsSubscribe` (mentions: Pump.fun Program ID).
- **Latencia:** <100ms (Modo `processed`).
- **Detección:** Captura eventos de `Create`, `Withdraw` (Graduación), `Buy` y `Sell` antes de que aparezcan en interfaces web.
- **Resiliencia:** Auto-reconexión con retry logic (máx 5 intentos).
- **Uso:** `cargo run -- scan`

### 🛡️ 4. Monitor 24/7 con Trailing Stop-Loss
Protección automática de posiciones.
- **Trailing SL:** Ajuste dinámico del stop-loss siguiendo el precio al alza.
- **Alertas Telegram:** Notificaciones instantáneas de cambios críticos.
- **Uso:** `cargo run`

---

## 📋 Comandos del Paddock

| Comando | Descripción | Estado |
|---------|-------------|--------|
| `./start_bot.sh` | Menú interactivo con todas las opciones | ✅ Operativo |
| `cargo run` | **Monitor Mode:** Vigilancia 24/7 con Trailing Stop-Loss. | ✅ Operativo |
| `cargo run -- buy --mint <M> --sol <S>` | **Execution Mode:** Compra inmediata desde terminal. | ⚠️ DNS Bloqueado |
| `cargo run -- scan` | **Telemetry Mode:** Scanner de eventos en Pump.fun. | ✅ Operativo |
| `python3 auto_audit.py <MINT>` | **Intelligence:** Auditoría técnica instantánea. | ✅ Operativo |

### Desde Telegram:
```
/buy <MINT> <SOL>   # Comprar token
/status             # Ver posiciones
/balance            # Ver balance
/targets            # Ver configuración
/help               # Ver ayuda
```

---

## 📊 Protocolo de Operación "Estándar Suizo"

1. **Detección:** El sensor WebSocket (`scan`) detecta una graduación.
2. **Auditoría:** Se lanza `auto_audit.py`. Si el veredicto es 🟢 APROBADO, se procede.
3. **Ejecución:** Se decide la entrada (manual o vía `buy` command cuando esté resuelto el DNS).
4. **Protección:** `The Chassis` toma el control con un Stop-Loss del -35% y Trailing Step de +30%.

---

## 📈 Roadmap de Ingeniería 2026

### ✅ Fase 2: Chasis Reforzado (Completado 2026-02-09)
- [x] Soporte CLI para comandos modulares.
- [x] Sensor de Logs WebSocket (Telemetría) con auto-reconnect.
- [x] Módulo Intelligence con veredicto automático.
- [x] Integración de Jupiter v6 en el motor de Rust.
- [x] Comando `/buy` en Telegram.
- [x] Script de arranque automatizado (`start_bot.sh`).
- [x] Guía de deployment para servidor VPS.

### 🚧 Fase 3: Soberanía Total (En Curso - Prioridad #1)
- [ ] **Raydium Direct Swap:** Eliminación de dependencia externa (Jupiter API).
  - Descubrimiento automático de Pools usando RPC.
  - Construcción de instrucciones de swap a bajo nivel.
  - Ver `docs/RAYDIUM_IMPLEMENTATION.md` para roadmap técnico.
- [ ] **Jito Bundles:** Ejecución atómica para garantizar entrada en el bloque 1.
- [ ] **Error Handling (Estándar 42):** Eliminación total de `unwrap()` y gestión de pánicos.

### 🔮 Fase 4: Inteligencia Artificial (Futuro)
- [ ] **gRPC / Geyser:** Migración de WebSockets a gRPC (Latencia de grado militar <20ms).
- [ ] **Dashboard Telemetría:** Interfaz visual estilo cockpit de F1.
- [ ] **ML Pattern Detection:** Detección de patrones de "Smart Money" usando históricos.

---

## 🖥️ Hosting en Servidor (Recomendado para 24/7)

Tu laptop es tu "Taller de Ingeniería", pero el bot debe vivir en un servidor para:
- **Uptime 24/7:** Sin depender de que tu laptop esté encendida.
- **Latencia Profesional:** Conexión directa a RPCs de Solana.
- **IP Estable:** Mayor confiabilidad con servicios RPC premium.

**Ver guía completa:** [`DEPLOYMENT.md`](DEPLOYMENT.md)

**Proveedores recomendados:**
- **Hetzner Cloud CX21:** €4.51/mes (2vCPU, 4GB RAM) - Alemania
- **DigitalOcean Droplet:** $6/mes (1vCPU, 1GB RAM) - NYC/SF
- **AWS Lightsail:** $5/mes (us-east-1) - Ultra latencia

---

## ⚠️ Disclaimer
Este sistema está diseñado por y para ingenieros con alta tolerancia al riesgo. La velocidad es nuestra ventaja, pero la disciplina es nuestra salvaguarda.

**Versión:** 1.1.0-luxury  
**Última Actualización:** 2026-02-09  
**Ingeniería:** Rubén | *MV Agusta Mindset* ⚡

