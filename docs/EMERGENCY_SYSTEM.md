# 🛡️ Emergency Exit System - The Chassis

## Overview
Sistema automático de protección de capital que monitorea posiciones activas y ejecuta exits rápidos cuando se cumplen condiciones críticas.

---

## ⚙️ Configuración

```rust
EmergencyConfig {
    max_loss_percent: -30.0,      // Stop loss al -30%
    min_sol_balance: 0.01,         // Alerta si SOL < 0.01
    min_asset_price: 0.000398,     // Precio mínimo del asset
    enabled: true,                 // Sistema activo
}
```

---

## 🎯 Triggers de Emergencia

### 1. **Stop Loss**
- **Condición:** `drawdown <= max_loss_percent`
- **Ejemplo:** Si entraste con 0.051 SOL y el drawdown alcanza -30%, se activa.
- **Acción:** Venta inmediata via Jito Bundle.

### 2. **Panic Sell**
- **Condición:** `current_price < min_asset_price`
- **Ejemplo:** $ICEBEAR cae por debajo de $0.000398
- **Acción:** Venta ultra-rápida antes de que el precio colapse más.

### 3. **Low Balance Alert** (Futuro)
- **Condición:** `sol_balance < min_sol_balance`
- **Ejemplo:** Quedan menos de 0.01 SOL (insuficiente para fees)
- **Acción:** Alertar para fondear la wallet.

---

## 📊 Ejemplo de Uso

### Añadir Posición al Monitoreo
```rust
let icebear_position = Position {
    token_mint: "86WM5NBUtRWTHULKrspS1TdzVFAcZ9buXsGRAiFDpump",
    entry_price: 0.0005687,
    amount_invested: 0.051,
    current_price: 0.000485,   // Actualizado en tiempo real
    current_value: 0.0435,     // Valor actual en SOL
};

emergency_monitor.add_position(icebear_position);
```

### Check de Emergencias
```rust
let alerts = emergency_monitor.check_emergencies();

for alert in alerts {
    match alert.alert_type {
        AlertType::StopLoss => {
            // Ejecutar venta inmediata
            execute_emergency_sell(&alert.token_mint).await?;
        },
        AlertType::PanicSell => {
            // Venta ultra-prioritaria (Jito Bundle con tip alto)
            execute_panic_sell(&alert.token_mint).await?;
        },
        _ => {}
    }
}
```

---

## 🚀 Integración Futura (v0.5.0)

### Jito Bundle Integration
```rust
async fn execute_emergency_sell(token_mint: &str) -> Result<()> {
    // 1. Crear transacción de venta
    let sell_ix = create_sell_instruction(token_mint, SlippageMode::Max)?;
    
    // 2. Empaquetar en Jito Bundle con prioridad ULTRA_HIGH
    let bundle = JitoBundle::new()
        .add_transaction(sell_ix)
        .set_tip(0.01) // 0.01 SOL tip para máxima prioridad
        .build()?;
    
    // 3. Enviar al leader de Jito
    bundle.send_and_confirm().await?;
    
    println!("✅ Emergency sell ejecutado en {} ms", elapsed);
    Ok(())
}
```

---

## 📈 Roadmap

### v0.4.0 (Actual)
- [x] Detección de Stop Loss
- [x] Detección de Panic Sell
- [x] Monitoreo de múltiples posiciones
- [x] Sistema de alertas

### v0.5.0 (Próximo)
- [ ] Integración con Jito Bundles
- [ ] Ejecución automática de exits
- [ ] Notificaciones Telegram
- [ ] Dashboard web en tiempo real

### v0.6.0 (Futuro)
- [ ] Machine Learning para predecir rug pulls
- [ ] Trailing Stop Loss dinámico
- [ ] Multi-wallet support

---

**Status:** 🟢 Operacional (Modo Alerta)  
**Próxima Acción:** Integrar ejecución real con Jito  
**Última Actualización:** 2026-02-08 02:17 CET
