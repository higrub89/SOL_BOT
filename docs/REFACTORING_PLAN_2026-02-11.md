# 🔧 Refactoring Plan - Institutional Grade Hardening
**Fecha:** 2026-02-11  
**Objetivo:** Transformar el bot de trading de prototipo funcional a sistema mission-critical

---

## 📋 Resumen Ejecutivo

Este plan aborda las 5 debilidades críticas identificadas en la auditoría externa:

1. ✅ **Estabilidad Crítica** - Eliminar `.unwrap()` y manejo robusto de errores
2. ✅ **Persistencia de Estado** - SQLite para posiciones y configuración dinámica
3. ✅ **Raydium Completo** - Descubrimiento on-chain de pools
4. ✅ **Dependencias Resilientes** - Fallbacks y circuit breakers
5. ✅ **Cálculos Seguros** - Validación estricta de datos financieros

---

## 🎯 Fase 1: Persistencia de Estado (CRÍTICO)
**Prioridad:** 🔴 CRÍTICA  
**Tiempo estimado:** 2-3 horas  
**Impacto:** Evita pérdida de datos en reinicios

### Tareas:
- [ ] Crear módulo `state_manager.rs`
- [ ] Diseñar schema SQLite para:
  - Posiciones activas (mint, entry_price, amount, current_sl, trailing_sl_state)
  - Historial de trades (signature, timestamp, type, pnl)
  - Configuración dinámica (última versión de targets.json)
- [ ] Implementar auto-save cada 5 segundos
- [ ] Migración automática desde `targets.json` en primer arranque
- [ ] Recovery automático al reiniciar

### Archivos a crear:
```
core/the_chassis/src/state_manager.rs
core/the_chassis/trading_state.db (generado automáticamente)
```

### Archivos a modificar:
```
core/the_chassis/src/lib.rs (integrar StateManager)
core/the_chassis/src/emergency.rs (persistir posiciones)
core/the_chassis/src/trailing_sl.rs (persistir estado TSL)
```

---

## 🎯 Fase 2: Robustez del Executor (ALTA)
**Prioridad:** 🟠 ALTA  
**Tiempo estimado:** 3-4 horas  
**Impacto:** Elimina crashes inesperados

### Estrategia de Refactoring:

#### 2.1 Executor V2 (`executor_v2.rs`)
**Problemas actuales:**
- Línea 106: `wallet_keypair.unwrap()` → puede paniquear
- Línea 172: `.parse::<f64>().unwrap_or(0.0)` → precio 0 es peligroso
- Línea 182: `.parse().unwrap_or(0.0)` → mismo problema
- Línea 236: `.parse().unwrap_or(0)` → cantidad 0 puede causar swap inválido
- Línea 313: `.parse::<f64>().unwrap_or(0.0)` → precio 0
- Línea 350: `.parse().unwrap_or(0.0)` → price impact 0
- Línea 390: `.await.unwrap_or_default()` → quote vacío es peligroso
- Línea 392: `.parse::<f64>().unwrap_or(0.0)` → precio 0
- Línea 411: `.parse().unwrap_or(0.0)` → price impact 0

**Solución:**
```rust
// ANTES (PELIGROSO):
let keypair = wallet_keypair.unwrap();

// DESPUÉS (SEGURO):
let keypair = wallet_keypair
    .ok_or_else(|| anyhow::anyhow!("Keypair requerido para ejecución real"))?;
```

```rust
// ANTES (PELIGROSO):
let sol_received = quote.out_amount.parse::<f64>().unwrap_or(0.0) / 1_000_000_000.0;

// DESPUÉS (SEGURO):
let sol_received = quote.out_amount
    .parse::<f64>()
    .context("Invalid out_amount from Jupiter API")?
    / 1_000_000_000.0;

// Validación adicional:
if sol_received <= 0.0 {
    anyhow::bail!("Invalid swap output: {} SOL", sol_received);
}
```

#### 2.2 Jupiter Client (`jupiter.rs`)
**Problemas actuales:**
- Línea 95-96: `.unwrap_or(0.0)` en cálculos de precio
- Línea 120: `.unwrap_or(0.0)` en price impact

**Solución:**
```rust
pub fn calculate_effective_price(&self, quote: &QuoteResponse) -> Result<f64> {
    let in_amount = quote.in_amount
        .parse::<f64>()
        .context("Invalid in_amount in quote")?;
    let out_amount = quote.out_amount
        .parse::<f64>()
        .context("Invalid out_amount in quote")?;

    if in_amount <= 0.0 {
        anyhow::bail!("Invalid input amount: {}", in_amount);
    }

    Ok(out_amount / in_amount)
}
```

#### 2.3 Scanner (`scanner.rs`)
**Problemas actuales:**
- Línea 21: `.unwrap()` al construir HTTP client
- Líneas 46-71: Múltiples `.unwrap_or(0.0)` en parsing de precios

**Solución:**
```rust
// Validar que el precio no sea 0 o negativo
if price_usd <= 0.0 {
    anyhow::bail!("Invalid price data from DexScreener: ${}", price_usd);
}

// Validar liquidez mínima
if liquidity_usd < 100.0 {
    anyhow::bail!("Liquidity too low: ${:.2}", liquidity_usd);
}
```

### Archivos a modificar:
```
core/the_chassis/src/executor_v2.rs
core/the_chassis/src/jupiter.rs
core/the_chassis/src/scanner.rs
core/the_chassis/src/raydium.rs
```

---

## 🎯 Fase 3: Raydium Pool Discovery (MEDIA-ALTA)
**Prioridad:** 🟡 MEDIA-ALTA  
**Tiempo estimado:** 4-5 horas  
**Impacto:** Autonomía completa para nuevos tokens

### Implementación:

#### 3.1 Completar `discover_pool_on_chain` en `raydium.rs`
```rust
fn discover_pool_on_chain(&self, base_mint: &str, quote_mint: &str) -> Result<PoolInfo> {
    println!("🔍 Buscando pool on-chain para {}/{}", base_mint, quote_mint);
    
    // Usar getProgramAccounts con filtros
    let filters = vec![
        // Filtro 1: Discriminator de Raydium AMM
        RpcFilterType::Memcmp(Memcmp::new_base58_encoded(0, &[...])),
        // Filtro 2: Base mint
        RpcFilterType::Memcmp(Memcmp::new_base58_encoded(400, base_mint)),
        // Filtro 3: Quote mint
        RpcFilterType::Memcmp(Memcmp::new_base58_encoded(432, quote_mint)),
    ];
    
    let config = RpcProgramAccountsConfig {
        filters: Some(filters),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            ..Default::default()
        },
        ..Default::default()
    };
    
    let accounts = self.rpc_client.get_program_accounts_with_config(
        &self.program_id,
        config,
    )?;
    
    // Parsear y validar pool
    // ...
}
```

#### 3.2 Cache automático
- Guardar pools descubiertos en `pools_cache.json` automáticamente
- TTL de 24 horas para re-validar pools

### Archivos a modificar:
```
core/the_chassis/src/raydium.rs
```

---

## 🎯 Fase 4: Resiliencia de APIs (MEDIA)
**Prioridad:** 🟡 MEDIA  
**Tiempo estimado:** 2-3 horas  
**Impacto:** Estabilidad en producción 24/7

### Estrategias:

#### 4.1 Circuit Breaker Pattern
```rust
pub struct CircuitBreaker {
    failure_count: AtomicU32,
    last_failure: Mutex<Option<Instant>>,
    threshold: u32,
    timeout: Duration,
}

impl CircuitBreaker {
    pub fn is_open(&self) -> bool {
        // Si hay muchos fallos recientes, "abrir" el circuito
        self.failure_count.load(Ordering::Relaxed) >= self.threshold
    }
    
    pub fn record_success(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
    }
    
    pub fn record_failure(&self) {
        self.failure_count.fetch_add(1, Ordering::Relaxed);
    }
}
```

#### 4.2 Retry con Exponential Backoff
```rust
async fn fetch_with_retry<T, F>(
    operation: F,
    max_retries: u32,
) -> Result<T>
where
    F: Fn() -> Future<Output = Result<T>>,
{
    let mut delay = Duration::from_millis(100);
    
    for attempt in 1..=max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_retries => {
                eprintln!("⚠️  Retry {}/{}: {}", attempt, max_retries, e);
                tokio::time::sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
            Err(e) => return Err(e),
        }
    }
    
    unreachable!()
}
```

#### 4.3 Fallback entre proveedores
```rust
// Prioridad: DexScreener → Jupiter Price API → On-chain directo
async fn get_price_with_fallback(&self, mint: &str) -> Result<f64> {
    // Intento 1: DexScreener (rápido)
    if let Ok(price) = self.dexscreener.get_price(mint).await {
        return Ok(price);
    }
    
    // Intento 2: Jupiter Price API
    if let Ok(price) = self.jupiter.get_price(mint).await {
        return Ok(price);
    }
    
    // Intento 3: Calcular desde pool on-chain (lento pero confiable)
    self.raydium.calculate_price_from_pool(mint).await
}
```

### Archivos a crear:
```
core/the_chassis/src/resilience.rs
```

### Archivos a modificar:
```
core/the_chassis/src/scanner.rs
core/the_chassis/src/jupiter.rs
```

---

## 🎯 Fase 5: Validación Financiera Estricta (ALTA)
**Prioridad:** 🟠 ALTA  
**Tiempo estimado:** 1-2 horas  
**Impacto:** Previene decisiones erróneas por datos corruptos

### Implementación:

```rust
pub struct PriceValidator;

impl PriceValidator {
    /// Valida que un precio sea razonable
    pub fn validate_price(price: f64, context: &str) -> Result<f64> {
        if price <= 0.0 {
            anyhow::bail!("{}: Precio inválido ({})", context, price);
        }
        
        if price.is_nan() || price.is_infinite() {
            anyhow::bail!("{}: Precio no numérico", context);
        }
        
        // Detectar precios absurdos (probablemente error de API)
        if price > 1_000_000_000.0 {
            anyhow::bail!("{}: Precio sospechosamente alto ({})", context, price);
        }
        
        Ok(price)
    }
    
    /// Valida cambio de precio razonable (anti-glitch)
    pub fn validate_price_change(
        old_price: f64,
        new_price: f64,
        max_change_percent: f64,
    ) -> Result<f64> {
        let change_pct = ((new_price - old_price) / old_price).abs() * 100.0;
        
        if change_pct > max_change_percent {
            anyhow::bail!(
                "Cambio de precio sospechoso: {:.2}% (límite: {:.2}%)",
                change_pct,
                max_change_percent
            );
        }
        
        Ok(new_price)
    }
}
```

### Archivos a crear:
```
core/the_chassis/src/validation.rs
```

---

## 📊 Métricas de Éxito

Después de completar este refactoring:

- ✅ **0 `.unwrap()` en código crítico** (executor, scanner, jupiter)
- ✅ **100% de posiciones persistidas** en SQLite
- ✅ **Raydium autónomo** (descubrimiento de pools)
- ✅ **3 niveles de fallback** para precios
- ✅ **Validación estricta** de todos los datos financieros
- ✅ **0 crashes** en 7 días de testing continuo

---

## 🚀 Orden de Ejecución Recomendado

1. **Fase 1** (Persistencia) - Base para todo lo demás
2. **Fase 2** (Robustez) - Eliminar puntos de fallo
3. **Fase 5** (Validación) - Complementa Fase 2
4. **Fase 4** (Resiliencia) - Mejora la experiencia
5. **Fase 3** (Raydium) - Feature avanzada

---

## 📝 Notas de Implementación

- Cada fase debe incluir **tests unitarios**
- Cada cambio debe ser **backward compatible** con `targets.json` existente
- Mantener **logs detallados** de cada decisión del sistema
- Implementar **dry-run mode** para testing de cada fase

---

**Próximo paso:** Empezar con Fase 1 - State Manager
