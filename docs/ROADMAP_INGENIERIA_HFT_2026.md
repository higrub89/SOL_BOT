# 🏎️ ROADMAP DE INGENIERÍA HFT 2026: THE CHASSIS v3.0

> **Objetivo:** Transformar el bot de trading en un Sistema de Ejecución Algorítmica Adaptativo (High-Frequency Trading) con tolerancia a fallos y gestión de riesgo institucional.
> **Filosofía:** "Safety-Critical Systems" (Sistemas Críticos de Seguridad) aplicado al trading de memecoins en Solana.

---

## 📅 FASE 1: EL CEREBRO (Decision Engine)
**Estado:** ✅ COMPLETADO
**Prioridad:** ALTA (Core Logic)

### 1.1 Momentum Sensor (El Corazón Matemático) ✅
- **Objetivo:** Detectar la derivada del precio/volumen (velocidad de cambio) con latencia O(1).
- **Implementación:**
  - [x] Estructura `MomentumSensor` con Ring Buffer circular (capacidad 12 puntos).
  - [x] Cálculo de pendiente usando **LWMA (Linear Weighted Moving Average)** para eficiencia.
  - [x] Métodos: `.update(price, timestamp)` y `.slope()`.
- **Thresholds Iniciales:**
  - Slope > +0.30/min: Señal fuerte (Trigger Dynamic Tip).
  - Slope < -0.20/min: Señal de venta/rechazo.

### 1.2 Pipeline de Decisión (Middleware) ✅
- **Objetivo:** Filtrar el 97% del "ruido" del mercado antes de ejecutar.
- **Implementación:**
  - [x] Trait `TradeFilter` para modularidad.
  - [x] Estructura `DecisionEngine` que encadena filtros en serie.
  - [x] Filtros Clave:
    1.  **Circuit Breaker Global:** Si PnL diario < -10%, apagar todo.
    2.  **Token Cooldown:** Bloquear token por 4-6h tras Stop Loss.
    3.  **Wash Trading:** Rechazar si Ratio (Unique Wallets / Tx) < 0.20.
    4.  **Narrative Correlation:** Limitar exposición a misma "moda" (max 20%).

### 1.3 Clasificación de Madurez (Maturity Stages) ✅
- **Objetivo:** Ajustar riesgo según la "edad" del token.
- **Implementación:**
  - [x] Enum `MaturityStage`:
    - **EarlyHighRisk (0-15 min):** Filtros extremos, tamaño posición 50%.
    - **MomentumCore (15-45 min):** Zona "Sweet Spot", tamaño posición 100%.
    - **LateReversal (>45 min):** Solo si volumen decay < 30%.
  - [x] Integración en `AutoBuyer`.

### 1.4 Actuadores Inteligentes ✅
- **Dynamic Jito Tip:** Ajuste de propina basado en urgencia (Momentum Slope).
- **Adaptive Slippage:** Ajuste de tolerancia basado en volatilidad.

---

## ⚡ FASE 2: LA EJECUCIÓN (Conexión de Sensores Reales)
**Duración Estimada:** 1 Semana (Sprint D)
**Prioridad:** ALTA (Data Integration)
**Estado:** 🚧 EN PROGRESO

### 2.1 Helius Sensor (Data Source)
- **Objetivo:** Alimentar el `TokenContext` con datos on-chain reales.
- **Implementación:**
  - Cliente gRPC/HTTP eficiente.
  - Parsing de `AccountInfo` para extraer autoridades (Mint/Freeze).

### 2.2 DexScreener Sensor (Market Data)
- **Objetivo:** Obtener precio, volumen y liquidez en tiempo real.
- **Implementación:**
  - Polling inteligente (respetando rate limits).
  - Cálculo de Unique Wallets para Wash Trading Filter.

### 2.3 Raydium Direct Executor (Optimization)
- **Objetivo:** Reducir latencia saltando el agregador Jupiter cuando sea posible.
- **Implementación:**
  - Finalizar integración de `RaydiumExecutor`.
  - Lógica de selección de ruta en `AutoBuyer`.

---

## 🛡️ FASE 3: LA DEFENSA (Risk Management Avanzado)
**Duración Estimada:** 3-4 Días (Sprint E)
**Prioridad:** MEDIA (Robustez)

### 3.1 Volatility Regime Switch
- **Objetivo:** No operar cuando la red Solana está saturada.
- **Implementación:**
  - Sensor de TPS y Slot Lag.
  - Si Slot Lag > 15 slots (aprox 6s), pausar nuevas entradas.
  - Si Failed Bundle Rate > 30%, pausar.

### 3.2 Multi-RPC Failover
- **Objetivo:** Redundancia de conexión.
- **Implementación:**
  - Pool de RPCs (Helius Premium + QuickNode Fallback).
  - Health Check activo cada 60s.
  - Cambio automático si latencia > 500ms o errores HTTP consecutivos.

---

## 📊 FASE 4: LA TELEMETRÍA (Black Box)
**Duración Estimada:** Continua
**Prioridad:** BAJA (Mejora a largo plazo)

### 4.1 False Negative Analysis
- **Objetivo:** Entender qué oportunidades perdimos.
- **Implementación:**
  - Script `operational/false_negatives.py`.
  - Analizar tokens rechazados que hicieron >200% en 1h.
  - Ajustar thresholds mensualmente.

### 4.2 PnL Attribution
- **Objetivo:** Entender qué filtro nos hace ganar/perder dinero.
- **Implementación:**
  - Taggear cada trade con los filtros que pasó (ej. "Entry: MomentumCore + HighSlope").
  - Reporte semanal de rendimiento por estrategia.
