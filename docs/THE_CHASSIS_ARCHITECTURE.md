# THE CHASSIS - Arquitectura de Trading Engine de Alto Rendimiento

**Fecha de Diseño:** 2026-02-06  
**Fase:** Diseño Conceptual (Pre-Implementación)  
**Objetivo:** Reducir latencia de decisión a <50ms y fricción de fees en >80%

---

## 🎯 Problema a Resolver

### Limitaciones Actuales (Python + Trojan Bot)
1. **Alta Fricción de Fees:**
   - Jito Tips: 0.001 SOL por transacción
   - Priority Fees: 0.005 SOL por transacción
   - Con 14 ciclos: ~0.084 SOL en fees (~$11-12 USD)
   - **Impacto:** Reduce ganancias reales en ~15-20%

2. **Latencia de Decisión:**
   - Python RPC calls: ~100-150ms
   - Trojan Bot processing: +50-100ms
   - **Total:** 150-250ms desde señal hasta ejecución
   - **Riesgo:** Perder entradas óptimas en mercados volátiles

3. **Dependencia de Terceros:**
   - Trojan Bot puede cambiar parámetros
   - Sin control sobre lógica de ejecución
   - No hay visibilidad del orderbook en tiempo real

---

## 🏗️ Arquitectura Propuesta

### Stack Tecnológico

```
┌─────────────────────────────────────────────────────────┐
│                    PRESENTATION LAYER                    │
│              (Terminal Dashboard - Python)               │
└─────────────────────────────────────────────────────────┘
                            ▲
                            │ gRPC
                            ▼
┌─────────────────────────────────────────────────────────┐
│                     DECISION ENGINE                      │
│                   (C++17 / Rust Core)                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │  Strategy   │  │   Risk Mgmt │  │  Portfolio  │     │
│  │  Executor   │  │   Module    │  │  Tracker    │     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
└─────────────────────────────────────────────────────────┘
                            ▲
                            │ Yellowstone gRPC
                            ▼
┌─────────────────────────────────────────────────────────┐
│                   DATA INGESTION LAYER                   │
│        (Yellowstone Geyser - Solana Block Stream)        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │  New Token   │  │   Liquidity  │  │  Whale Txs   │  │
│  │  Listener    │  │   Monitor    │  │   Tracker    │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
└─────────────────────────────────────────────────────────┘
                            ▲
                            │ WebSocket
                            ▼
┌─────────────────────────────────────────────────────────┐
│                   EXECUTION LAYER                        │
│              (Jito Bundle Manager - Rust)                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │  Transaction │  │  Bundle      │  │  MEV         │  │
│  │  Builder     │  │  Assembler   │  │  Protection  │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
└─────────────────────────────────────────────────────────┘
                            ▲
                            │ JSON-RPC
                            ▼
                    ┌───────────────┐
                    │ Solana Network│
                    └───────────────┘
```

---

## 🔧 Módulos Core

### 1. Yellowstone Geyser Listener (C++/Rust)
**Propósito:** Streaming de bloques en tiempo real desde Solana  
**Latencia Objetivo:** <20ms desde bloque hasta procesamiento

**Funcionalidades:**
- Subscribe a nuevos tokens (detección de mint)
- Monitor de cambios de liquidez en Raydium/Orca
- Tracking de transacciones de wallets específicas (Smart Money)
- Filtrado de eventos relevantes (reduce noise en 99%)

**Tecnología:**
- gRPC streaming (Yellowstone)
- Protobuf para serialización
- Lock-free queues para high-throughput

### 2. Decision Engine (C++17)
**Propósito:** Ejecución de estrategias con latencia ultra-baja

**Subcomponents:**
- **Strategy Executor:** Implementa lógica de "Golden Rules"
  - LP Burned check
  - Mint Authority check
  - Holder distribution analysis
  - Liquidity threshold validation
  
- **Risk Management Module:**
  - Dynamic position sizing (Kelly Criterion)
  - Real-time PnL tracking
  - Auto-stop loss triggers
  - Max drawdown protection

- **Portfolio Tracker:**
  - Rent cost tracking
  - Fee accumulation alerts
  - Cross-position correlation

### 3. Jito Bundle Manager (Rust)
**Propósito:** Construcción y envío de bundles atómicos

**Beneficios vs. Jito Tips actuales:**
- **Fee Reduction:** 1 bundle tip vs. N transaction tips
  - Actual: 0.001 SOL × 14 = 0.014 SOL
  - Con Bundles: 0.001 SOL × 1 = 0.001 SOL
  - **Ahorro: ~93%**

- **MEV Protection:**
  - Bundles son atómicos (todo o nada)
  - No pueden ser sandwiched
  - Ejecución en el mismo bloque garantizada

**Tecnología:**
- Jito Block Engine API
- Transaction simulation antes de envío
- Retry logic con exponential backoff

### 4. Terminal Dashboard (Python - FastAPI + Rich)
**Propósito:** Interfaz humana para monitoreo y control

**Features:**
- Real-time PnL tracking
- Network health monitoring (latency, fees)
- Audit checklist integration
- Manual override controls
- Historical performance charts

---

## 📊 Métricas de Éxito

### Performance Targets
| Métrica | Actual (Python) | Target (The Chassis) | Mejora |
|---------|----------------|---------------------|--------|
| **Latencia de Decisión** | 150-250ms | <50ms | 3-5x |
| **Fee por Operación** | 0.084 SOL (14 cycles) | 0.001 SOL (1 bundle) | 84x |
| **Rugs Detectados** | 2/2 (100%) | >95% | Mantener |
| **False Positives** | Desconocido | <10% | TBD |

### Cost-Benefit Analysis
**Desarrollo Estimado:** 40-60 horas  
**Ahorro en Fees (1 mes):** ~0.3-0.5 SOL (~$40-70 USD)  
**ROI:** Break-even en 2-3 meses de trading activo

---

## 🚀 Roadmap de Implementación

### Fase 0: Proof of Concept (Semana 1-2)
- [x] Documentar arquitectura ✅
- [ ] Setup de entorno de desarrollo C++/Rust
- [ ] Hello World con Yellowstone Geyser
- [ ] Test de latencia baseline

### Fase 1: Data Layer (Semana 3-4)
- [ ] Implementar Geyser listener básico
- [ ] Parser de eventos de mint/liquidity
- [ ] Logger de datos en tiempo real
- [ ] Integración con Helius RPC como fallback

### Fase 2: Decision Engine (Semana 5-6)
- [ ] Port de "Golden Rules" a C++
- [ ] Implementar risk management module
- [ ] Unit tests para cada regla
- [ ] Benchmark de latencia de decisión

### Fase 3: Execution Layer (Semana 7-8)
- [ ] Jito Bundle builder en Rust
- [ ] Transaction signing con keypair
- [ ] Simulation y dry-run mode
- [ ] Error handling y retries

### Fase 4: Integration (Semana 9-10)
- [ ] Terminal dashboard con FastAPI
- [ ] End-to-end testing en Devnet
- [ ] Migración gradual desde Trojan Bot
- [ ] Live testing con capital mínimo (0.1 SOL)

### Fase 5: Production Hardening (Semana 11-12)
- [ ] Monitoring y alertas
- [ ] Logging distribuido
- [ ] Backups y disaster recovery
- [ ] Auditoría de seguridad

---

## 🔐 Consideraciones de Seguridad

### Wallet Management
- **Burner Wallets:** Generación programática con rotación diaria
- **Key Storage:** Encrypted keystore con password protection
- **Separation of Concerns:** Trading wallet vs. Main wallet

### Code Security
- **Dependency Audit:** Scan de vulnerabilidades con cargo-audit
- **Input Validation:** Sanitización de todas las entradas externas
- **Rate Limiting:** Protección contra DoS en APIs

### Operational Security
- **Dry-Run Mode:** Simular operaciones sin ejecutar
- **Max Position Size:** Hard limit en código (no configurable)
- **Emergency Stop:** Kill switch accesible por hotkey

---

## 📚 Referencias Técnicas

### Solana Development
- [Yellowstone Geyser gRPC](https://docs.helius.dev/solana-rpc-nodes/geyser-enhanced-websockets)
- [Solana Cookbook](https://solanacookbook.com/)
- [Anchor Framework](https://www.anchor-lang.com/)

### Jito MEV
- [Jito Block Engine](https://jito-labs.gitbook.io/mev)
- [Bundle Transactions Guide](https://jito-foundation.gitbook.io/mev/searcher-resources/bundles)

### High-Performance C++
- [Lock-Free Programming](https://preshing.com/20120612/an-introduction-to-lock-free-programming/)
- [Zero-Cost Abstractions](https://doc.rust-lang.org/book/ch10-00-generics.html)

---

## 🎯 Next Steps

### Decisión Requerida
**¿Comenzamos con la Fase 0 (Proof of Concept) HOY?**

Si aceptas:
1. Setup de repositorio `core/` con estructura C++/Rust
2. Instalación de dependencias (gRPC, Protobuf, Rust toolchain)
3. Hello World con Yellowstone Geyser
4. Benchmark de latencia contra Python actual

**Tiempo estimado:** 2-3 horas  
**Output:** Primer commit en `core/` con POC funcional

**¿Proceder?** 🚀

---

**Versión:** 0.1 (Draft)  
**Autor:** Ruben + AI Architect  
**Status:** Awaiting Go/No-Go Decision
