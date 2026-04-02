# 🏎️ THE CHASSIS - Solana Trading Engine v2.0

> [!IMPORTANT]
> **🤖 PROJECT AGENT-READY**: Este repositorio es un ecosistema autocontenido para asistentes de IA.
> Si eres un agente, lee **[AGENTS.md](file:///home/ruben/Workspace/defi/bot_trading/AGENTS.md)** para inicializa tus herramientas (MCP) y habilidades (Skills) locales antes de operar.

**Autor:** Rubén (rhiguita) | 42 Madrid  
**Entorno:** Ubuntu 24.04 LTS / GCP `solana-bot-v1`  
**Filosofía:** Ingeniería de precisión HFT (Estilo MV Agusta / Ferrari)  
**Stack Principal:** Rust (Core Async), Python (Intelligence), Helius (Infrastructure)

---

## 🏎️ El Concepto: "The Chassis" v2.0.0-HFT

Este sistema es un **chasis institucional de alto rendimiento** diseñado para la ejecución determinista en Solana. Bajo los estándares de **42 Madrid**, hemos evolucionado hacia una arquitectura asíncrona de **zero-allocation** en los hot-paths, garantizando latencias sub-100µs.

## 🏗️ Arquitectura Agent-Native (Hiperluxury Standard)

```
bot_trading/
├── core/                # 🚀 MOTOR HFT (Zero-Alloc Async Executor)
├── intelligence/        # 🧠 ML & BACKTESTING (Strategy Engine)
├── operational/         # 📊 DEVOPS & TELEMETRÍA (GCP / WIF)
├── mcp/                 # 🛠️ ORQUESTACIÓN IA (Local MCP Servers)
├── skills/              # ⚡ HABILIDADES IA (Atomic Agent Tasks)
├── AGENTS.md            # 📡 MISSION CONTROL (Agent Discovery)
└── mcpServers.json      # 🧩 MANIFIESTO MAESTRO (IA Tooling)
```

## 📋 Comandos del Paddock

| Comando | Descripción | Estado |
|---------|-------------|--------|
| `cargo run` | **Monitor Mode:** Vigilancia 24/7 con Trailing Stop-Loss. | ✅ Operativo |
| `make test` | **Audit Mode:** Unit & Integration tests. | ✅ Operativo |
| `make backtest` | **Intelligence:** Simulación de estrategias ML. | ✅ Operativo |

---

## 🛡️ Protocolo de Operación "Estándar Suizo"

1. **Detección:** El sensor Geyser/WS detecta liquidez o señales de ML.
2. **Auditoría:** Se activa el `StrategyEngine` para validar slippage y rentabilidad.
3. **Ejecución:** Swap atómico vía `executor_v2` (Zero-Alloc).
4. **Protección:** `The Chassis` activa el Trailing Stop-Loss dinámico.

---

## 🔮 Roadmap v2.x
- [x] **WIF Integration:** Autenticación segura sin llaves JSON.
- [x] **Agent-Ready Repo:** Descubrimiento automático de herramientas.
- [ ] **Jito Bundles:** Ejecución en el bloque 0.
- [ ] **Ultra-Low Latency:** <50µs jitter p99.9.

---
**Ingeniería:** Rubén | *Antigravity AI Assisted* ⚡
