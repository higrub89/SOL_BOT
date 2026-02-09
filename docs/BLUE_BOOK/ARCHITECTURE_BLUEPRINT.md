# 🏗️ THE BLUE BOOK - Architecture Blueprint

**Proyecto:** The Chassis - Solana Trading Engine  
**Versión:** 2.0.0 (Framework Institucional)  
**Fecha:** 2026-02-09

---

## 1. Visión General del Sistema

The Chassis es un framework de ejecución de alta frecuencia para trading en Solana. Diseñado con principios de sistemas críticos: **velocidad**, **resiliencia** y **soberanía técnica**.

### 1.1 Arquitectura en Capas

```
┌─────────────────────────────────────────────────────────────┐
│                    OPERATOR LAYER (Humano)                  │
│        Telegram Bot | CLI | Config Files (targets.json)     │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────┴────────────────────────────────────┐
│                  ORCHESTRATION LAYER (Rust)                 │
│   ┌─────────────┐   ┌──────────────┐   ┌────────────────┐  │
│   │  Emergency  │   │   Monitor    │   │   Trailing SL  │  │
│   │   System    │   │   Engine     │   │    Manager     │  │
│   └─────────────┘   └──────────────┘   └────────────────┘  │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────┴────────────────────────────────────┐
│                 EXECUTION LAYER (Trait-based)               │
│   ┌──────────────────────────────────────────────────────┐  │
│   │             Executor Trait (Abstraction)             │  │
│   └────┬─────────────────────────────────────────┬───────┘  │
│        │                                         │           │
│   ┌────▼────────┐                       ┌────────▼──────┐   │
│   │   Jupiter   │                       │    Raydium    │   │
│   │  Executor   │                       │   Executor    │   │
│   │ (Liquidity) │                       │    (Speed)    │   │
│   └─────────────┘                       └───────────────┘   │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────┴────────────────────────────────────┐
│                 INTELLIGENCE LAYER (Python via gRPC)        │
│   ┌─────────────┐   ┌──────────────┐   ┌────────────────┐  │
│   │    Auto     │   │  Smart Money │   │   Liquidity    │  │
│   │   Audit     │   │   Tracker    │   │    Analysis    │  │
│   └─────────────┘   └──────────────┘   └────────────────┘  │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────┴────────────────────────────────────┐
│                   DATA LAYER (Solana + APIs)                │
│   ┌──────────────┐   ┌──────────────┐   ┌───────────────┐  │
│   │   Helius     │   │  DexScreener │   │  RugCheck API │  │
│   │ RPC + Geyser │   │     API      │   │               │  │
│   └──────────────┘   └──────────────┘   └───────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Flujo de Datos: Desde Detection hasta Execution

### 2.1 Ciclo Completo de un Trade

```
┌──────────────────────────────────────────────────────────────────┐
│                    1. DETECTION PHASE                            │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  User Input (CLI/Telegram)                                       │
│       │                                                          │
│       ▼                                                          │
│  ┌─────────────────┐                                            │
│  │ Token Discovery │  ← DexScreener API (nuevos tokens)         │
│  └────────┬────────┘                                            │
│           │                                                      │
│           ▼                                                      │
│  ┌─────────────────┐                                            │
│  │  Initial Checks │                                            │
│  │  - Liquidity?   │                                            │
│  │  - Volume?      │                                            │
│  └────────┬────────┘                                            │
└───────────┼──────────────────────────────────────────────────────┘
            │
            │ PASS
            ▼
┌──────────────────────────────────────────────────────────────────┐
│                    2. AUDIT PHASE (gRPC)                         │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Rust (Chassis)                       Python (Intelligence)     │
│       │                                        │                │
│       │  ──[gRPC Request]────────────────────▶ │                │
│       │    {token_mint: "EPjF..."}             │                │
│       │                                        ▼                │
│       │                              ┌──────────────────┐       │
│       │                              │  Auto Audit      │       │
│       │                              │  - RugCheck API  │       │
│       │                              │  - Holder Cluster│       │
│       │                              │  - LP Analysis   │       │
│       │                              └────────┬─────────┘       │
│       │                                       │                │
│       │  ◀────[gRPC Response]─────────────────┤                │
│       │    {score: 85, verdict: "SAFE"}       │                │
│       ▼                                                         │
│  ┌─────────────────┐                                            │
│  │ Decision Logic  │                                            │
│  │ Score >= 70?    │                                            │
│  └────────┬────────┘                                            │
└───────────┼──────────────────────────────────────────────────────┘
            │
            │ APPROVED
            ▼
┌──────────────────────────────────────────────────────────────────┐
│                   3. EXECUTION PHASE (Buy)                       │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌────────────────┐                                             │
│  │ Executor Trait │  ◀── FallbackExecutor decides              │
│  └───────┬────────┘                                             │
│          │                                                      │
│    ┌─────┴──────┐                                               │
│    │ Is Raydium │                                               │
│    │  Healthy?  │                                               │
│    └─────┬──────┘                                               │
│          │                                                      │
│     YES  │  NO                                                  │
│    ┌─────▼──────┐         ┌──────────────┐                     │
│    │  Raydium   │         │   Jupiter    │                     │
│    │  Executor  │         │   Executor   │                     │
│    └─────┬──────┘         └──────┬───────┘                     │
│          │                       │                              │
│          │   1. get_quote()      │                              │
│          ├──────────────────────▶│                              │
│          │   2. execute_swap()   │                              │
│          ├──────────────────────▶│                              │
│          │                       │                              │
│          │   ┌───────────────────▼───────────────┐              │
│          │   │   Helius RPC (Premium Endpoint)   │              │
│          │   │   - sendTransaction()             │              │
│          │   │   - confirmTransaction()          │              │
│          │   └───────────────────┬───────────────┘              │
│          │                       │                              │
│          │  ◀────Signature────────                              │
│          ▼                                                      │
│  ┌──────────────────┐                                           │
│  │  SwapExecution   │  → Log to telemetry                      │
│  │  {signature,     │                                           │
│  │   latency_ms: 420│                                           │
│  │   slippage: 0.5%}│                                           │
│  └──────────┬───────┘                                           │
└─────────────┼────────────────────────────────────────────────────┘
              │
              ▼
┌──────────────────────────────────────────────────────────────────┐
│                  4. MONITORING PHASE                             │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌────────────────┐         ┌─────────────────┐                 │
│  │ Position Opened│────────▶│  Emergency      │                 │
│  │ Entry: $0.00005│         │  Monitor        │                 │
│  └────────────────┘         │  - Trailing SL  │                 │
│         │                   │  - Stop Loss    │                 │
│         │                   └────────┬────────┘                 │
│         │                            │                          │
│    [Every 5s]                   [Trigger?]                      │
│         │                            │                          │
│         ▼                            ▼                          │
│  ┌─────────────────┐        ┌───────────────┐                  │
│  │ Price Scanner   │        │  Auto Sell    │                  │
│  │ (DexScreener)   │        │  (executor)   │                  │
│  └────────┬────────┘        └───────────────┘                  │
│           │                                                     │
│           ▼                                                     │
│  ┌─────────────────┐                                            │
│  │ Update Position │                                            │
│  │ Current: $0.00006│                                           │
│  │ PnL: +20%       │                                            │
│  └─────────────────┘                                            │
└──────────────────────────────────────────────────────────────────┘
```

---

## 3. Componentes Técnicos Clave

### 3.1 Executor Trait (Polimorfismo)

**Archivo:** `src/executor_trait.rs`

```rust
#[async_trait]
pub trait Executor {
    fn name(&self) -> &str;
    async fn get_quote(...) -> Result<Quote>;
    async fn execute_swap(...) -> Result<SwapExecution>;
    async fn is_healthy() -> bool;
    async fn avg_latency_ms() -> u64;
}
```

**Implementaciones:**
- `JupiterExecutor`: Para liquidez agregada
- `RaydiumExecutor`: Para velocidad pura (en desarrollo)
- `FallbackExecutor`: Combina ambos con failover automático

---

### 3.2 gRPC Bridge (Rust ↔ Python)

**Archivo Proto:** `proto/chassis.proto`

```protobuf
service ChassisService {
    rpc GetTokenAudit (AuditRequest) returns (AuditResponse);
}

message AuditRequest {
    string token_mint = 1;
}

message AuditResponse {
    string verdict = 1;  // "SAFE", "CAUTION", "RUG"
    int32 score = 2;     // 0-100
    double lp_locked_pct = 3;
}
```

**Flujo:**
1. Rust detecta nuevo token
2. Llama `chassis_service.get_token_audit(mint)`
3. Python ejecuta auditoría (RugCheck, DexScreener, análisis de wallets)
4. Devuelve score y veredicto
5. Rust decide si comprar

---

### 3.3 Observability System

**Archivo:** `src/observability.rs`

**Stack:**
- `tracing`: Structured logging
- `tracing-subscriber`: Log formatting
- `tracing-appender`: Archivo rotativo diario

**Niveles:**
- TRACE: Debugging extremo
- DEBUG: Desarrollo
- INFO: Producción (default)
- WARN: Anomalías recuperables
- ERROR: Fallos críticos

**Macros:**
```rust
log_swap!("Raydium", signature, 420, 0.5);
log_audit!(mint, 85, "SAFE");
log_error!("EXECUTOR", error, "context");
```

---

### 3.4 Persistencia (The Black Box)

**Tecnología:** SQLite con `sqlx`

**Schema (propuesto):**
```sql
CREATE TABLE positions (
    id INTEGER PRIMARY KEY,
    token_mint TEXT NOT NULL,
    entry_price REAL NOT NULL,
    entry_time INTEGER NOT NULL,
    amount_sol REAL NOT NULL,
    stop_loss_pct REAL NOT NULL,
    status TEXT NOT NULL, -- OPEN, CLOSED, EMERGENCY_SOLD
    exit_price REAL,
    exit_time INTEGER,
    pnl_pct REAL
);

CREATE TABLE trades (
    id INTEGER PRIMARY KEY,
    position_id INTEGER,
    type TEXT NOT NULL, -- BUY, SELL
    signature TEXT UNIQUE NOT NULL,
    amount_in REAL NOT NULL,
    amount_out REAL NOT NULL,
    executor TEXT NOT NULL, -- Jupiter, Raydium
    latency_ms INTEGER NOT NULL,
    slippage_pct REAL NOT NULL,
    timestamp INTEGER NOT NULL,
    FOREIGN KEY (position_id) REFERENCES positions(id)
);

CREATE TABLE audit_results (
    id INTEGER PRIMARY KEY,
    token_mint TEXT NOT NULL,
    score INTEGER NOT NULL,
    verdict TEXT NOT NULL,
    lp_locked_pct REAL,
    timestamp INTEGER NOT NULL
);
```

**Ventajas:**
- **ACID Compliance:** Transacciones atómicas
- **Recovery:** El bot puede reiniciarse y retomar posiciones
- **Analytics:** Queries SQL para análisis histórico

---

## 4. Principios de Diseño

### 4.1 Soberanía Técnica
> "El que controla la abstracción, controla el sistema."

- **No lock-in:** Podemos cambiar de Jupiter a Raydium en segundos
- **No single point of failure:** Fallback automático entre DEXs
- **Control total:** Acceso directo a pools de Raydium

### 4.2 Resiliencia
- **Retry logic:** Reintentos con exponential backoff
- **Health checks:** Monitoreo constante de servicios externos
- **Graceful degradation:** Si gRPC falla, opera sin auditoría pero alerta

### 4.3 Observabilidad
- **Log everything:** Cada trade, cada error, cada latencia
- **Structured logging:** Formato parseavel para análisis automático
- **Telemetría de hiperlujo:** Logs dignos de F1

---

## 5. Evolución del Sistema (Roadmap)

### Fase 1: Alpha Production (Actual)
- ✅ Jupiter integration
- ✅ Emergency system
- ✅ Telegram notifications
- ✅ Basic monitoring

### Fase 2: Framework Institucional (En Curso)
- 🚧 Trait-based executors
- 🚧 Raydium direct integration
- 🚧 gRPC intelligence bridge
- 🚧 SQLite persistence
- 🚧 Premium observability

### Fase 3: High-Frequency Trading
- ⏳ Yellowstone Geyser (100-300ms latency reduction)
- ⏳ Jito Bundles (anti-frontrunning)
- ⏳ ML-based prediction
- ⏳ Smart Money tracking

### Fase 4: Soberanía Total
- ⏳ Zero dependencia de APIs externas
- ⏳ Custom Solana node
- ⏳ Mesh network con otros traders
- ⏳ Modo "Shadow Race" (testing sin capital)

---

## 6. Métricas de Éxito

| Métrica | Target Fase 2 | Actual |
|---------|---------------|--------|
| Latencia total (quote → confirm) | <1000ms | ~2000ms |
| Slippage real | <1.5% | <2% |
| Uptime del bot | >99% | ~95% |
| Win Rate | >40% | 50% |
| Trades documentados | 50+ | 2 |

---

## 7. Referencias

- **Solana Docs:** https://docs.solana.com
- **Raydium SDK:** https://github.com/raydium-io/raydium-sdk
- **Jupiter API:** https://station.jup.ag/docs
- **gRPC Rust:** https://github.com/hyperium/tonic
- **Tracing:** https://docs.rs/tracing

---

**Fin del Architecture Blueprint**  
**Próximo Documento:** `SECURITY_VAULT.md` (Manejo de claves privadas)
