# 🔄 State Manager Integration Guide
**Fecha:** 2026-02-11  
**Objetivo:** Integrar el State Manager en el flujo de monitoreo existente

---

## 📋 Cambios Necesarios en `lib.rs`

### 1. Inicializar State Manager al arrancar

```rust
// En run_monitor_mode(), después de cargar AppConfig:

use state_manager::{StateManager, PositionState};

// Inicializar State Manager
let state_manager = Arc::new(StateManager::new("trading_state.db")?);

println!("📊 STATE MANAGER:");
let stats = state_manager.get_stats()?;
println!("   • Posiciones activas: {}", stats.active_positions);
println!("   • Trades históricos:  {}", stats.total_trades);
println!("   • PnL total:          {:.4} SOL", stats.total_pnl_sol);
```

### 2. Migrar posiciones desde targets.json

```rust
// Después de cargar emergency_monitor:

println!("\n🔄 Migrando posiciones a State Manager...");

for target in &app_config.targets {
    if !target.active { continue; }
    
    // Verificar si ya existe en DB
    if let Some(existing) = state_manager.get_position(&target.mint)? {
        println!("   ✓ {} ya existe en DB (entry: ${})", target.symbol, existing.entry_price);
        continue;
    }
    
    // Crear nueva posición
    let position = PositionState {
        id: None,
        token_mint: target.mint.clone(),
        symbol: target.symbol.clone(),
        entry_price: target.entry_price,
        amount_sol: target.amount_sol,
        current_price: target.entry_price,
        stop_loss_percent: target.stop_loss_percent,
        trailing_enabled: target.trailing_enabled,
        trailing_distance_percent: target.trailing_distance_percent,
        trailing_activation_threshold: target.trailing_activation_threshold,
        trailing_highest_price: None,
        trailing_current_sl: None,
        active: true,
        created_at: Utc::now().timestamp(),
        updated_at: Utc::now().timestamp(),
    };
    
    state_manager.upsert_position(&position)?;
    println!("   ✓ {} migrado a DB", target.symbol);
}
```

### 3. Actualizar precios en el loop principal

```rust
// En el loop de monitoreo, después de obtener el precio:

match scanner.get_token_price(&target.mint).await {
    Ok(price) => {
        // Actualizar precio en State Manager
        let _ = state_manager.update_position_price(&target.mint, price.price_usd);
        
        // ... resto de la lógica existente
    }
}
```

### 4. Persistir estado de Trailing SL

```rust
// Cuando el Trailing SL se actualiza:

if let Some(tsl) = trailing_monitors.get_mut(&target.symbol) {
    if tsl.update(price.price_usd) {
        // Guardar nuevo estado en DB
        let _ = state_manager.update_trailing_sl(
            &target.mint,
            tsl.highest_price,
            tsl.current_sl_percent,
        );
    }
}
```

### 5. Registrar trades ejecutados

```rust
// Después de ejecutar una venta:

match sell_result {
    Ok(swap_result) => {
        println!("✅ Venta automática completada: {}", swap_result.signature);
        
        // Calcular PnL
        let pnl_sol = swap_result.output_amount - target.amount_sol;
        let pnl_percent = (pnl_sol / target.amount_sol) * 100.0;
        
        // Registrar trade en DB
        let trade = TradeRecord {
            id: None,
            signature: swap_result.signature.clone(),
            token_mint: target.mint.clone(),
            symbol: target.symbol.clone(),
            trade_type: "EMERGENCY_SELL".to_string(),
            amount_sol: target.amount_sol,
            tokens_amount: swap_result.input_amount,
            price: price.price_usd,
            pnl_sol: Some(pnl_sol),
            pnl_percent: Some(pnl_percent),
            route: swap_result.route.clone(),
            price_impact_pct: swap_result.price_impact_pct,
            timestamp: Utc::now().timestamp(),
        };
        
        let _ = state_manager.record_trade(&trade);
        
        // Cerrar posición en DB
        let _ = state_manager.close_position(&target.mint);
        
        // Notificar
        let _ = telegram_clone.send_message(
            &format!(
                "✅ Venta automática de {} completada.\\n\
                 Signature: {}\\n\
                 PnL: {:.4} SOL ({:.2}%)",
                target.symbol, swap_result.signature, pnl_sol, pnl_percent
            ),
            true
        ).await;
    }
}
```

---

## 🔍 Comandos de Telegram Nuevos

Agregar a `telegram_commands.rs`:

### `/positions` - Ver posiciones activas

```rust
"positions" => {
    let positions = state_manager.get_active_positions()?;
    
    if positions.is_empty() {
        return Ok("No hay posiciones activas.".to_string());
    }
    
    let mut msg = "📊 **POSICIONES ACTIVAS**\\n\\n".to_string();
    
    for pos in positions {
        let dd = ((pos.current_price - pos.entry_price) / pos.entry_price) * 100.0;
        let value = (pos.amount_sol / pos.entry_price) * pos.current_price;
        
        msg.push_str(&format!(
            "**{}**\\n\
             Entry: ${:.8}\\n\
             Current: ${:.8}\\n\
             Drawdown: {:.2}%\\n\
             Value: {:.4} SOL\\n\\n",
            pos.symbol, pos.entry_price, pos.current_price, dd, value
        ));
    }
    
    Ok(msg)
}
```

### `/history [N]` - Ver últimos N trades

```rust
"history" => {
    let limit = parts.get(1)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10);
    
    let trades = state_manager.get_trade_history(limit)?;
    
    if trades.is_empty() {
        return Ok("No hay trades registrados.".to_string());
    }
    
    let mut msg = format!("📜 **ÚLTIMOS {} TRADES**\\n\\n", trades.len());
    
    for trade in trades {
        let pnl_str = if let Some(pnl) = trade.pnl_sol {
            format!("{:.4} SOL ({:.2}%)", pnl, trade.pnl_percent.unwrap_or(0.0))
        } else {
            "N/A".to_string()
        };
        
        msg.push_str(&format!(
            "**{}** - {}\\n\
             Price: ${:.8}\\n\
             PnL: {}\\n\
             Signature: `{}`\\n\\n",
            trade.symbol,
            trade.trade_type,
            trade.price,
            pnl_str,
            &trade.signature[..16]
        ));
    }
    
    Ok(msg)
}
```

### `/stats` - Estadísticas generales

```rust
"stats" => {
    let stats = state_manager.get_stats()?;
    let (total_pnl, trade_count) = state_manager.calculate_total_pnl()?;
    
    let avg_pnl = if trade_count > 0.0 {
        total_pnl / trade_count
    } else {
        0.0
    };
    
    Ok(format!(
        "📊 **ESTADÍSTICAS GENERALES**\\n\\n\
         Posiciones activas: {}\\n\
         Total trades: {}\\n\
         PnL total: {:.4} SOL\\n\
         PnL promedio: {:.4} SOL\\n",
        stats.active_positions,
        stats.total_trades,
        total_pnl,
        avg_pnl
    ))
}
```

---

## 🧪 Testing del State Manager

### Test 1: Crear posición y recuperarla

```bash
# En el directorio del proyecto
cd core/the_chassis
cargo test test_position_lifecycle -- --nocapture
```

### Test 2: Simular ciclo completo

```rust
#[tokio::test]
async fn test_full_trading_cycle() {
    let manager = StateManager::new(":memory:").unwrap();
    
    // 1. Crear posición
    let position = PositionState {
        token_mint: "TEST123".to_string(),
        symbol: "TEST".to_string(),
        entry_price: 0.001,
        amount_sol: 1.0,
        current_price: 0.001,
        stop_loss_percent: -20.0,
        trailing_enabled: true,
        trailing_distance_percent: 5.0,
        trailing_activation_threshold: 10.0,
        trailing_highest_price: Some(0.0011),
        trailing_current_sl: Some(-15.0),
        active: true,
        created_at: Utc::now().timestamp(),
        updated_at: Utc::now().timestamp(),
        id: None,
    };
    
    manager.upsert_position(&position).unwrap();
    
    // 2. Actualizar precio
    manager.update_position_price("TEST123", 0.0012).unwrap();
    
    // 3. Actualizar TSL
    manager.update_trailing_sl("TEST123", 0.0012, -10.0).unwrap();
    
    // 4. Registrar venta
    let trade = TradeRecord {
        signature: "SIG123".to_string(),
        token_mint: "TEST123".to_string(),
        symbol: "TEST".to_string(),
        trade_type: "SELL".to_string(),
        amount_sol: 1.0,
        tokens_amount: 1000.0,
        price: 0.0012,
        pnl_sol: Some(0.2),
        pnl_percent: Some(20.0),
        route: "Raydium".to_string(),
        price_impact_pct: 0.5,
        timestamp: Utc::now().timestamp(),
        id: None,
    };
    
    manager.record_trade(&trade).unwrap();
    
    // 5. Cerrar posición
    manager.close_position("TEST123").unwrap();
    
    // 6. Verificar
    let stats = manager.get_stats().unwrap();
    assert_eq!(stats.active_positions, 0);
    assert_eq!(stats.total_trades, 1);
    assert_eq!(stats.total_pnl_sol, 0.2);
}
```

---

## ⚠️ Consideraciones Importantes

1. **Backup automático**: El archivo `trading_state.db` debe incluirse en backups regulares
2. **Migración gradual**: El sistema puede coexistir con `targets.json` durante la transición
3. **Recovery**: Si la DB se corrompe, el bot puede reconstruir desde `targets.json`
4. **Performance**: SQLite es síncrono, pero las operaciones son tan rápidas que no afectan el loop

---

## 🚀 Próximos Pasos

1. ✅ State Manager creado
2. ⏳ Integrar en `lib.rs` (siguiente paso)
3. ⏳ Agregar comandos de Telegram
4. ⏳ Testing en producción con dry-run
5. ⏳ Activar persistencia completa

---

**Nota**: Una vez integrado, el bot será **stateful** y podrá reiniciarse sin perder información crítica.
