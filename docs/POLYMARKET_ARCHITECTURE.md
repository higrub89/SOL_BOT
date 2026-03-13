# 🎯 Polymarket Bot — Arquitectura de Predicción

**Módulo:** `polymarket/`  
**Versión:** 0.1.0  
**Stack:** Rust (gRPC Engine) + Python (Strategy Prototyping)

---

## Concepto

Bot automático de compra/venta de posiciones en **mercados de predicción** de Polymarket.
Misma filosofía que The Chassis (SOL_BOT), pero adaptado al dominio de probabilidades.

**En vez de:** comprar/vender SOL a un precio  
**Aquí:** comprar/vender shares de YES/NO en eventos con probabilidad implícita

---

## Arquitectura

```
polymarket/
├── proto/
│   └── polymarket.proto       # Definición gRPC del servicio PolymarketBot
├── build.rs                   # Compilador de .proto → código Rust
├── Cargo.toml                 # Dependencias del crate
├── scripts/
│   └── strategy_client.py     # Cliente Python para estrategias rápidas
└── src/
    ├── lib.rs                 # Exports del módulo
    ├── bin/
    │   └── main.rs            # CLI entry point (markets/positions/serve/config)
    ├── config.rs              # Configuración (API, gRPC, riesgo)
    ├── types.rs               # Tipos de dominio (Market, Position, Order)
    ├── client.rs              # Cliente HTTP/WebSocket para Polymarket API
    ├── strategy.rs            # Trait PredictionStrategy + EdgeDetector
    ├── risk.rs                # Gestión de riesgo (oráculo, deadlines, limites)
    └── generated/             # Código generado por protobuf
        └── mod.rs
```

---

## Servicio gRPC

```protobuf
service PolymarketBot {
  rpc GetMarkets (GetMarketsRequest) returns (GetMarketsResponse);
  rpc PlaceOrder (PlaceOrderRequest) returns (PlaceOrderResponse);
  rpc CancelOrder (CancelOrderRequest) returns (CancelOrderResponse);
  rpc GetPositions (GetPositionsRequest) returns (GetPositionsResponse);
  rpc StreamMarketUpdates (MarketStreamRequest) returns (stream MarketUpdate);
}
```

### Diferencias clave vs ChassisService (SOL_BOT)

| Concepto | SOL_BOT (Solana) | Polymarket Bot |
|----------|------------------|----------------|
| Activo | SOL / SPL tokens | Shares YES/NO |
| Precio | Precio de mercado | Probabilidad implícita (0.0–1.0) |
| Ejecución | Jupiter/Raydium swap | Polymarket CLOB order |
| Riesgo | Volatilidad, liquidez | + Oráculo, resolución, deadline |
| Estrategia | Momentum, entry/exit | Edge detection, market making |

---

## Flujo de Trading

```
Polymarket API (REST/WebSocket)
    ↓
PolymarketClient (HTTP + WS)
    ↓
PredictionStrategy (análisis de edge/probabilidad)
    ↓
RiskManager (validación de límites)
    ↓
PlaceOrder via gRPC
    ↓
Estado y PnL actualizados
```

---

## Estrategias

### EdgeDetector (incluida)
Compara la probabilidad interna estimada contra el precio de mercado.
Si detecta un "edge" superior al umbral configurado, sugiere operar.

```rust
pub trait PredictionStrategy: Debug + Send + Sync {
    fn name(&self) -> &str;
    fn initialize(&self) -> Result<()>;
    fn analyze(&self, snapshot: &MarketSnapshot) -> Result<PredictionAction>;
    fn estimate_probability(&self, snapshot: &MarketSnapshot) -> Option<f64>;
}
```

### Futuras estrategias posibles
- **Market Making:** Proveer liquidez bid/ask en mercados activos
- **Arbitrage:** Entre Polymarket y otras fuentes de probabilidad
- **Sentiment:** Integración con noticias/social media para ajustar probabilidades

---

## Gestión de Riesgo

El `RiskManager` evalúa cada orden antes de ejecutarla:

1. ✅ Mercado activo
2. ✅ Límite de posiciones no excedido
3. ✅ Tamaño dentro del máximo por posición
4. ✅ Exposición total al mercado dentro de límites
5. ✅ Liquidez suficiente en el mercado

---

## Comandos CLI

```bash
# Ver mercados disponibles
cargo run -p polymarket_bot -- markets --limit 20

# Ver posiciones abiertas
cargo run -p polymarket_bot -- positions

# Arrancar servidor gRPC
cargo run -p polymarket_bot -- serve

# Ver configuración
cargo run -p polymarket_bot -- config
```

---

## Configuración (Variables de Entorno)

| Variable | Default | Descripción |
|----------|---------|-------------|
| `POLYMARKET_REST_URL` | `https://clob.polymarket.com` | API REST |
| `POLYMARKET_WS_URL` | `wss://ws-subscriptions-clob.polymarket.com/ws/market` | WebSocket |
| `POLYMARKET_API_KEY` | (vacío) | API key de Polymarket |
| `POLYMARKET_GRPC_ADDR` | `[::1]:50052` | Dirección del servidor gRPC |
| `POLYMARKET_MAX_POSITION_USDC` | `100.0` | Máximo USDC por posición |

---

## Roadmap

### ✅ Fase 1: Fundación (Actual)
- [x] Definición de .proto para PolymarketBot
- [x] Tipos de dominio (Market, Position, Order)
- [x] Cliente HTTP para Polymarket API
- [x] Trait PredictionStrategy + EdgeDetector
- [x] Módulo de gestión de riesgo
- [x] CLI básico
- [x] Integración en workspace

### 🚧 Fase 2: Ejecución (Próximo)
- [ ] Servidor gRPC completo (tonic::Server)
- [ ] WebSocket streaming de precios
- [ ] Firma de transacciones (Polygon/Polymarket)
- [ ] Persistencia de estado (SQLite)
- [ ] Python gRPC client con proto bindings

### 🔮 Fase 3: Inteligencia
- [ ] Estrategias avanzadas (Market Making, Arbitraje)
- [ ] Backtesting con datos históricos de Polymarket
- [ ] Integración de noticias/sentiment analysis
- [ ] Dashboard de monitoreo
