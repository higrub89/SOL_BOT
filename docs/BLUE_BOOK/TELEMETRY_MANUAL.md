# 📘 THE BLUE BOOK - Manual de Telemetría

**Proyecto:** The Chassis - Solana Trading Engine  
**Versión:** 2.0.0 (Framework Institucional)  
**Fecha:** 2026-02-09  
**Clasificación:** Documentación Técnica de Grado Institucional

---

## 1. Filosofía de Observabilidad

> "Si no está en los logs, no existió."

En sistemas de trading de alta frecuencia, la telemetría es tan crítica como el código mismo. Un log bien estructurado puede:
- **Diagnosticar fallos** en microsegundos
- **Auditar comportamiento** de algoritmos
- **Cumplir con regulaciones** (si escalamos a institucional)
- **Optimizar rendimiento** con métricas precisas

---

## 2. Niveles de Log

### 2.1 TRACE (Solo Desarrollo)
Debugging extremo. Cada byte, cada paso del algoritmo.

**Ejemplo:**
```
[TRACE][RAYDIUM] Deserializing pool account | Offset: 400 | Bytes: [0x2a, 0x1b...]
```

**Cuándo usar:** Nunca en producción. Solo para ingeniería profunda.

---

### 2.2 DEBUG
Información de diagnóstico útil para entender el flujo.

**Ejemplo:**
```
[DEBUG][EXECUTOR] Quote obtained | DEX: Jupiter | Input: 1.5 SOL | Output: 1500000 tokens
```

**Cuándo usar:** Durante desarrollo y staging.

---

### 2.3 INFO (Producción)
Eventos importantes del sistema. **Este es el nivel estándar de producción.**

**Formato Estándar:**
```
[YYYY-MM-DD HH:MM:SS.mmm][INFO][MODULE] Event | Field1: Value1 | Field2: Value2
```

**Ejemplos:**
```
[2026-02-09 22:15:01.423][INFO][EXECUTOR-RAYDIUM] Swap Success | TX: 5ghZ... | Latency: 420ms | Slippage: 0.5%
[2026-02-09 22:15:05.128][INFO][AUDIT] Token Analyzed | Mint: EPjF... | Score: 85 | Verdict: SAFE
[2026-02-09 22:15:10.001][INFO][EMERGENCY] SL Triggered | Symbol: $DOOM | DD: -12.5% | Action: AUTO_SELL
```

**Cuándo usar:** Para registrar TODOS los eventos de negocio críticos.

---

### 2.4 WARN
Situaciones anómalas pero recuperables.

**Ejemplos:**
```
[WARN][EXECUTOR-JUPITER] API Slow | Latency: 1520ms | Threshold: 1000ms | Action: Switching to Raydium
[WARN][LIQUIDITY] LP Drop Detected | Token: $PEPE | Drop: -25% | Alert: TELEGRAM_SENT
```

**Cuándo usar:** Cuando el sistema se auto-recupera pero queremos investigar después.

---

### 2.5 ERROR
Errores que requieren atención. El sistema NO se auto-recuperó.

**Ejemplos:**
```
[ERROR][EXECUTOR-RAYDIUM] Transaction Failed | TX: 4hY... | Error: Insufficient SOL | Balance: 0.001
[ERROR][GRPC] Audit Service Unavailable | Retries: 3 | Status: CONNECTION_REFUSED
```

**Cuándo usar:** Cuando se requiere intervención manual o el sistema está degradado.

---

## 3. Módulos del Sistema

### 3.1 EXECUTOR-RAYDIUM
Ejecución de swaps directos en Raydium.

**Eventos:**
- `Quote Requested`
- `Pool Discovered`
- `Swap Submitted`
- `Swap Success`
- `Swap Failed`

**Métricas críticas:**
- `latency_ms`: Tiempo desde quote hasta confirmación
- `slippage_pct`: Slippage real vs esperado
- `gas_paid`: Fees totales en SOL

---

### 3.2 EXECUTOR-JUPITER
Ejecución via aggregator Jupiter.

**Eventos:**
- `API Called`
- `Route Calculated`
- `Swap Executed`

**Métricas críticas:**
- `api_latency_ms`
- `route_hops`: Número de DEXs en la ruta (1-5)
- `price_impact_pct`

---

### 3.3 AUDIT
Sistema de auditoría de tokens (gRPC Python ↔ Rust).

**Eventos:**
- `Token Submitted`
- `gRPC Request Sent`
- `Score Received`
- `Verdict Applied`

**Campos:**
- `token_mint`: Address del token
- `score`: 0-100
- `verdict`: SAFE | CAUTION | RUG
- `lp_locked_pct`: % de LP bloqueado
- `top_10_holders_pct`: % en top 10 wallets

---

### 3.4 EMERGENCY
Sistema de Stop Loss y gestión de riesgo.

**Eventos:**
- `Position Opened`
- `Price Updated`
- `SL Triggered`
- `Manual Override`

**Métricas críticas:**
- `drawdown_pct`: Pérdida actual desde entrada
- `sl_threshold`: Límite configurado
- `action`: ALERT | AUTO_SELL | MANUAL_REQUIRED

---

### 3.5 LIQUIDITY
Monitor de liquidez en tiempo real.

**Eventos:**
- `Snapshot Captured`
- `LP Drop Detected`
- `Volume Surge Detected`

**Métricas:**
- `liquidity_usd`
- `volume_24h`
- `change_pct`: Cambio desde última snapshot

---

## 4. Formato de Log de "Hiperlujo"

### 4.1 Estructura Base
```
[TIMESTAMP][LEVEL][MODULE] Event | Field1: Value1 | Field2: Value2 | ...
```

### 4.2 Ejemplos Reales

**Swap Exitoso:**
```
[2026-02-09 22:15:01.423][INFO][EXECUTOR-RAYDIUM] Swap Success | TX: 5ghZp2K...3Ld4 | Input: 1000000000 lamports | Output: 1500000 tokens | Latency: 420ms | Slippage: 0.5% | Gas: 0.00015 SOL
```

**SL Activado:**
```
[2026-02-09 22:18:45.001][INFO][EMERGENCY] SL Triggered | Symbol: $DOOM | Entry: $0.000042 | Current: $0.000037 | DD: -12.5% | Threshold: -10.0% | Action: AUTO_SELL | TX: 7kPq...9Xz2
```

**Auditoría Completada:**
```
[2026-02-09 22:10:12.500][INFO][AUDIT] Token Analyzed | Mint: EPjFW...pV2s | Score: 85 | Verdict: SAFE | LP_Locked: 95.2% | Top10: 18.5% | RugCheck: GOOD | Response_Time: 2.1s
```

---

## 5. Archivo de Logs Rotativo

### 5.1 Ubicación
```
/home/ruben/Automatitation/bot_trading/operational/logs/chassis.log
```

### 5.2 Rotación
- **Frecuencia:** Diaria (rotación a medianoche UTC)
- **Nomenclatura:** `chassis.log.YYYY-MM-DD`
- **Retención:** 30 días (después se archiva o elimina)

### 5.3 Formato JSON (Producción Avanzada)
Para parsing automático por herramientas de análisis:

```json
{
  "timestamp": "2026-02-09T22:15:01.423Z",
  "level": "INFO",
  "module": "EXECUTOR-RAYDIUM",
  "event": "Swap Success",
  "fields": {
    "tx": "5ghZp2K...3Ld4",
    "latency_ms": 420,
    "slippage_pct": 0.5,
    "gas_sol": 0.00015
  }
}
```

---

## 6. Macros de Conveniencia

### 6.1 `log_swap!`
```rust
log_swap!(
    "Raydium-AMM-v4",
    "5ghZp2K...3Ld4",
    420,
    0.5
);
```

### 6.2 `log_audit!`
```rust
log_audit!(
    "EPjFW...pV2s",
    85,
    "SAFE"
);
```

### 6.3 `log_error!`
```rust
log_error!(
    "EXECUTOR-RAYDIUM",
    error,
    "Failed to build swap instruction"
);
```

---

## 7. Monitoreo en Tiempo Real

### 7.1 Tail de Logs
```bash
tail -f operational/logs/chassis.log | grep "ERROR"
```

### 7.2 Filtrado por Módulo
```bash
cat operational/logs/chassis.log | grep "\[EXECUTOR-RAYDIUM\]"
```

### 7.3 Análisis de Latencias
```bash
cat operational/logs/chassis.log | grep "Swap Success" | awk -F'|' '{print $4}' | grep -oP '\d+(?=ms)'
```

---

## 8. Cumplimiento y Auditoría

### 8.1 Registros Obligatorios
Para cumplir con estándares institucionales, **TODOS** estos eventos deben ser logueados:

- ✅ Cada swap ejecutado (buy/sell)
- ✅ Cada activación de Stop Loss
- ✅ Cada auditoría de token
- ✅ Cada fallo de transacción
- ✅ Cambios en configuración (targets.json)

### 8.2 Retención de Logs
- **Mínimo:** 30 días
- **Recomendado:** 365 días para análisis de ML
- **Archivado:** Comprimir logs antiguos con `gzip`

---

## 9. Ejemplos de Sesión Completa

```
[2026-02-09 22:10:00.000][INFO][SYSTEM] The Chassis Started | Version: 2.0.0 | Mode: PRODUCTION
[2026-02-09 22:10:00.050][INFO][SYSTEM] Observability Initialized | Log Level: INFO | Dir: operational/logs
[2026-02-09 22:10:01.200][INFO][WALLET] Balance Checked | Address: HF2UG... | SOL: 0.162
[2026-02-09 22:10:05.001][INFO][EMERGENCY] Positions Loaded | Count: 2 | Symbols: [$DOOM, $PEPE]
[2026-02-09 22:10:12.500][INFO][AUDIT] Token Analyzed | Mint: EPjFW... | Score: 85 | Verdict: SAFE
[2026-02-09 22:12:30.123][DEBUG][EXECUTOR-RAYDIUM] Quote Requested | Input: SOL | Output: EPjFW... | Amount: 0.05 SOL
[2026-02-09 22:12:30.543][INFO][EXECUTOR-RAYDIUM] Quote Obtained | Expected: 125000 tokens | Impact: 0.8%
[2026-02-09 22:12:31.001][INFO][EXECUTOR-RAYDIUM] Swap Submitted | TX: 5ghZp...
[2026-02-09 22:12:31.421][INFO][EXECUTOR-RAYDIUM] Swap Success | TX: 5ghZp... | Latency: 420ms | Slippage: 0.5%
[2026-02-09 22:15:00.000][INFO][MONITOR] Price Updated | Symbol: $DOOM | Price: $0.000037 | DD: -12.5%
[2026-02-09 22:15:00.100][WARN][EMERGENCY] SL Proximity | Symbol: $DOOM | Distance: 2.5% | Alert: TELEGRAM_SENT
[2026-02-09 22:18:45.001][INFO][EMERGENCY] SL Triggered | Symbol: $DOOM | Action: AUTO_SELL
[2026-02-09 22:18:46.234][INFO][EXECUTOR-JUPITER] Emergency Sell Executed | TX: 7kPq... | SOL_Recovered: 0.044
```

---

## 10. Próximos Pasos (Observabilidad Avanzada)

### 10.1 Integración con Prometheus
Para métricas en tiempo real y dashboards.

### 10.2 Alertas Automáticas
- Si `latency_ms > 1000`: Email al administrador
- Si `ERROR` count \u003e 5 en 1min: Telegram crítico

### 10.3 Machine Learning sobre Logs
Entrenar modelos para predecir fallos antes de que ocurran.

---

**Fin del Manual de Telemetría**  
**Próximo Documento:** `ARCHITECTURE_BLUEPRINT.md` (Diagrama de flujo Helius → RPC → Ejecución)
