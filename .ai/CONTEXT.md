# bot_trading — AI Context (The Chassis / SOL_BOT)

## Stack

### Lenguajes y runtime
- Rust 2021 (motor principal de trading)
- Python 3 (auditoría/inteligencia y utilidades)
- Bash (operación y despliegue)
- SQLite (persistencia de estado de trading)

### Workspace Rust
- `Cargo.toml` (root) con miembros:
  - `core` (`the_chassis`)
  - `intelligence` (`intelligence_rs`)
- Perfil `release` optimizado:
  - `opt-level = 3`
  - `lto = "fat"`
  - `codegen-units = 1`
  - `panic = "abort"`
  - `strip = "symbols"`
  - `incremental = false`

### Dependencias clave (core)
- Solana: `solana-client`, `solana-sdk`, `solana-program` (2.0.0)
- Tokens/DEX: `spl-token`, `spl-associated-token-account`
- Async: `tokio` (full), `tokio-stream`
- Red/API: `reqwest`, `tokio-tungstenite`
- gRPC/proto: `tonic`, `prost`, `tonic-build`
- Persistencia: `rusqlite`, `deadpool-sqlite`
- Observabilidad: `tracing`, `tracing-subscriber`
- Control remoto: Telegram commands + notificador

### Entrypoints reales
- Binario: `core/src/bin/main.rs` (`the_chassis_app`)
- Entrypoint app: `the_chassis::run()` en `core/src/lib.rs`
- CLI:
  - `buy --mint <MINT> --sol <SOL> [--slippage <bps>]`
  - `autobuy --mint <MINT> [--sol <SOL>] [--symbol <SYM>] [--monitor <bool>]`
  - `scan`
  - `monitor` (modo por defecto)

---

## Arquitectura

### Estructura del repositorio
- `core/`: ejecución HFT (scanner, decision engine, ejecución, riesgo, estado, Telegram, telemetry)
- `intelligence/`: estrategia/backtesting (Rust) + scripts de soporte (Python)
- `operational/`: scripts operativos y artefactos (`logs/`, `audits/`)
- `docs/`: manuales, estado, roadmap y guías operativas
- `.ai/`: contexto para agentes IA

### Pipeline operativo (monitor mode)
1. Carga `.env` + `settings.json`
2. Inicializa:
   - `WalletMonitor`
   - `StateManager` (SQLite + WAL)
   - `EmergencyMonitor`
   - `PriceFeed`
   - `TelemetryServer` (`127.0.0.1:9001`)
   - `CommandHandler` de Telegram
3. Recupera posiciones DB y ejecuta **Ghost Position Purge** contra estado on-chain
4. Arranca loop desacoplado:
   - `StrategyEngine` -> emite `ExecutionCommand`
   - `ExecutionRouter` -> ejecuta BUY/TP/SL/PANIC
5. Mantiene tareas de hibernación automática y notificación

### Mapa de módulos clave (`core/src`)
- `lib.rs`: orquestación completa del sistema
- `executor_v2.rs`: lógica de ejecución de trades
- `auto_buyer.rs`: compra inteligente (raydium/jupiter pathing)
- `raydium.rs`: swap directo Raydium (en evolución)
- `jupiter.rs`: integración con Jupiter
- `jito.rs`: integración con Jito
- `price_feed.rs`: hub de precios multi-fuente
- `sensors/helius.rs`: análisis on-chain (authorities, holders, edad, concentración)
- `state_manager.rs`: schema SQLite, migraciones, queries
- `emergency.rs`: SL/panic checks
- `telemetry_server.rs`: websocket telemetry + comandos UI (`HIBERNATE`, `PANIC_ALL`)
- `telegram/commands/*`: comandos remotos y parsing
- `engine/*`: filtros, estrategia, actuadores y routing de ejecución

### Inteligencia (`intelligence/`)
- `src/strategy_engine.rs`: trait `Strategy`, `TradeAction`, `SellReason`
- `src/backtesting.rs`: simulador de backtesting
- `scripts/auto_audit.py`: auditoría RugCheck + DexScreener, reporte en `operational/audits`
- `scripts/chassis_buy.py`: ejecución de compra vía Jupiter desde Python

---

## Configuración y secretos

### Variables de entorno (`.env.example`)
- Requeridas:
  - `HELIUS_API_KEY`
  - `WALLET_ADDRESS`
  - `WALLET_PRIVATE_KEY`
- Opcionales:
  - `JUPITER_API_KEY`
  - `TELEGRAM_BOT_TOKEN`
  - `TELEGRAM_CHAT_ID`
  - `GEYSER_ENDPOINT`
  - `DEXSCREENER_INTERVAL_SEC`
  - `MAX_LATENCY_MS`

### Config runtime (`settings.json`)
- `global_settings.min_sol_balance`
- `global_settings.jito_tip_lamports`
- `global_settings.monitor_interval_sec`
- `global_settings.auto_execute`

### Resolución de secretos
- `core/src/wallet.rs`:
  - `get_env_or_secret(name)`: ENV primero
  - fallback a GCP Secret Manager (`gcloud secrets versions access latest`)
- Mapping especial:
  - `WALLET_PRIVATE_KEY` -> `CHASSIS_WALLET_KEY`

---

## Persistencia y estado

### Base de datos
- Archivo: `trading_state.db`
- Driver: `rusqlite` + pool `deadpool-sqlite`
- WAL habilitado al iniciar `StateManager`

### Tablas
- `positions`: posición activa, SL/trailing/TP1/TP2, flags de estado
- `trades`: historial con `trade_type`, `route`, `price_impact_pct`, `fee_sol`
- `execution_audits`: trazabilidad de decisión vs ejecución
- `metadata`: metadatos operativos (ej. offset Telegram)

### Comportamientos críticos
- Migraciones idempotentes (`ALTER TABLE`) al iniciar
- Purga de posiciones fantasma (DB vs balance real on-chain)

---

## Observabilidad y control operacional

### Observabilidad
- Inicialización en `observability` (dev/prod)
- `TelemetryServer` emite ticks a 1 Hz por WebSocket
- Logs de runtime en scripts/containers (`logs/`)

### Telegram
- Polling manual con offset persistido en SQLite
- Whitelist por `chat_id` autorizado
- Comandos de control, ejecución y analytics en `telegram/commands`

### Hibernación
- Flag global `HIBERNATION_MODE`
- Se activa automáticamente por balance bajo
- Puede activarse también por comando UI/Telegram

---

## Comandos útiles para agentes IA

### Calidad y pruebas
- `cargo check --workspace`
- `cargo test --workspace`
- `make check`
- `make build`
- `make test`
- `make lint`
- `make format`

### Ejecución
- `cargo run -p the_chassis`
- `cargo run -p the_chassis -- scan`
- `cargo run -p the_chassis -- buy --mint <MINT> --sol <SOL>`
- `cargo run -p the_chassis -- autobuy --mint <MINT> --sol <SOL>`

### Scripts
- `./start_bot.sh` (launcher interactivo)
- `./bot_manager.sh start|stop|restart|status|logs`
- `python3 check_apis.py` (health checks APIs)
- `./operational/scripts/trading_session.sh` (sesión operativa)

### Deploy
- Docker multi-stage (`Dockerfile`)
- `docker-compose.yml` con capacidades RT (`SYS_NICE`, `rtprio`)
- CI/CD GCP: `.github/workflows/gcp-deploy.yml`

---

## MCPs activos (proyecto)
- No se identifican MCPs de proyecto explícitos en el repo para runtime de trading.
- Este archivo (`.ai/CONTEXT.md`) es la base de contexto para agentes.
- MCPs globales del entorno pueden existir fuera del repo.

---

## Reglas específicas

### Convenciones observables
- Documentación y comentarios mayoritariamente en español.
- Arquitectura asíncrona desacoplada por canales `tokio::mpsc`.
- Persistencia SQLite como componente crítico de resiliencia.
- Manejo de secretos con patrón ENV-first + GCP fallback.

### Reglas prácticas para editar sin romper
- Si cambias `core/src/lib.rs`, validar wiring completo:
  - `PriceFeed`
  - `StateManager`
  - `StrategyEngine`
  - `ExecutionRouter`
  - Telegram/Telemetry
- Si tocas execution/risk:
  - revisar impacto en `TradeRecord` y `record_trade`
  - mantener errores explícitos (`anyhow::Result` + `Context`)
- Si tocas configuración:
  - reflejar cambios en `.env.example` + docs
- Si tocas scripts bash:
  - mantener estilo defensivo (`set -euo pipefail`) en scripts nuevos/modificados

---

## Estado actual

### Estado técnico verificado
- `cargo check --workspace` ✅
- `cargo test --workspace` ✅ (36 passed, 4 ignored)

### Capacidades implementadas
- Modos operativos: monitor / scan / buy / autobuy
- Persistencia de posiciones y trades
- Telemetría WS para UI
- Control remoto por Telegram
- Sensores de precio/on-chain y pipeline de decisión

### Áreas activas / evolución
- Raydium direct swap (roadmap + código en progreso)
- Endurecimiento adicional de manejo de errores en rutas legacy
- Ajustes continuos de latencia, ejecución y observabilidad

### Riesgos/gotchas para agentes
- Credenciales reales son obligatorias para ejecución real.
- Parte de `docs/` y scripts incluye rutas históricas/legacy.
- Priorizar siempre comportamiento observable en código (`core/src`, `intelligence/src`) sobre documentos narrativos antiguos.

---

## Guía rápida: dónde tocar según tarea
- Nuevo comando CLI -> `core/src/lib.rs`
- Nuevo comando Telegram -> `core/src/telegram/commands/*`
- Estrategia/filtros -> `core/src/engine/*` + `intelligence/src/strategy_engine.rs`
- Persistencia DB -> `core/src/state_manager.rs`
- Fuentes de precio/fallback -> `core/src/price_feed.rs` + `core/src/sensors/*`
- Riesgo/exits -> `core/src/emergency.rs`, `core/src/trailing_sl.rs`, `core/src/engine/router.rs`
- Deploy/infra -> `Dockerfile`, `docker-compose.yml`, `.github/workflows/gcp-deploy.yml`, `setup_gcp.sh`

---

## Nota final para agentes
Este repo combina componentes productivos y piezas de roadmap/legacy.  
Para decisiones de implementación, tomar como fuente de verdad:
1. código ejecutable actual,
2. configuración real (`.env` + `settings.json`),
3. test suite del workspace.
