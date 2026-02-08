# 🚀 Plan de Ejecución: Finalizar Auto-Sell Integration

## 🎯 Objetivo
Completar la integración de Jupiter para permitir auto-sell cuando se rompa el Stop Loss.

## ⚡ Ruta Rápida (Opción Recomendada)

### Estrategia: API-Only Approach
En vez de construir transacciones localmente, llamamos a Jupiter API directamente y usamos su SDK simplificado.

### Pasos:

#### 1. Simplificar el Executor (15 min)
Crear `src/executor_simple.rs` que:
- Llama a Jupiter Quote API
- Muestra la mejor ruta
- Retorna la URL de transacción
- El usuario ejecuta en Jupiter UI (semi-automático)

```rust
pub async fn execute_emergency_sell_url(
    &self,
    token_mint: &str,
    wallet: &str,
) -> Result<String> {
    // 1. Obtener quote
    let quote = self.jupiter.get_quote(...).await?;
    
    // 2. Generar URL de Jupiter UI
    let url = format!(
        "https://jup.ag/swap/{}-SOL?amount={}",
        token_mint,
        quote.in_amount
    );
    
    // 3. Abrir en navegador automáticamente
    println!("🔗 Abriendo Jupiter en navegador...");
    webbrowser::open(&url)?;
    
    Ok(url)
}
```

**Ventajas:**
- ✅ No requiere manejo de private keys en código
- ✅ No requiere dependencias complejas de Solana SDK
- ✅ Usa la UI de Jupiter (más confiable)
- ✅ Implementable en 15 minutos

**Desventajas:**
- ⚠️ Requiere confirmación manual del usuario
- ⚠️ No es 100% automático, pero es 90% automático

---

#### 2. Integrar con el Monitor (10 min)
```rust
// En main.rs
if dd <= target.stop_loss_percent {
    if app_config.global_settings.auto_execute {
        println!("⚡ GENERANDO TRANSACCIÓN DE EMERGENCIA...");
        
        match executor.execute_emergency_sell_url(&target.mint, &wallet_addr).await {
            Ok(url) => {
                println!("✅ Transacción preparada en Jupiter");
                println!("🔗 URL: {}", url);
                println!("⏱️  CONFIRMA LA VENTA EN LA VENTANA DEL NAVEGADOR");
            }
            Err(e) => {
                eprintln!("❌ Error: {}", e);
                println!("⚠️  FALLBACK: VENDER EN TROJAN");
            }
        }
    }
}
```

---

#### 3. (Opcional) Webhook a Telegram (20 min extra)
Si quieres ir un paso más allá:
```rust
// src/telegram.rs
pub async fn send_emergency_alert(
    token: &str,
    message: &str,
    jupiter_url: &str,
) -> Result<()> {
    let bot_token = env::var("TELEGRAM_BOT_TOKEN")?;
    let chat_id = env::var("TELEGRAM_CHAT_ID")?;
    
    let text = format!(
        "🚨 EMERGENCY ALERT!\n\n{}\n\n🔗 {}",
        message, jupiter_url
    );
    
    // Enviar mensaje
    // ... código de reqwest a Telegram API
    
    Ok(())
}
```

---

## 🏗️ Alternativa: Implementación Completa (60-90 min)

Si prefieres tener el 100% automático (firmado y enviado sin intervención):

### Opción 1: Downgrade a Solana SDK 1.17
```toml
[dependencies]
solana-client = "1.17"
solana-sdk = "1.17"
spl-token = "4.0"
```

### Opción 2: Usar Jupiter Swap SDK de Rust
```bash
# Agregar al Cargo.toml
jupiter-swap-api-client = "0.1"
```

Pero esto requiere:
1. Implementar firma de transacciones
2. Manejo seguro de private keys
3. Sistema de confirmación de transacciones
4. Manejo de errores de red

---

## 🎲 Mi Recomendación

**Para la próxima sesión:**

1. **Implementar la Ruta Rápida (API-Only)** primero
   - 25 minutos de desarrollo
   - Funcional el mismo día
   - Semi-automático pero muy efectivo

2. **Si funciona bien, añadir Telegram Webhook**
   - Te avisa instantáneamente
   - Recibes el link de Jupiter en el móvil
   - Confirmas desde donde estés

3. **Dejar la Implementación Completa para Fase 3**
   - Cuando tengas más tiempo
   - Con más tests de seguridad
   - Con wallet encryption implementado

---

## 📋 Próximos Pasos Inmediatos

```bash
# 1. Limpia el build actual
cd /home/ruben/Automatitation/bot_trading/core/the_chassis
cargo clean

# 2. Crea una rama para el experimento
git checkout -b feature/executor-simple

# 3. Implementa executor_simple.rs (anexo abajo)

# 4. Actualiza main.rs para usar executor_simple

# 5. Test y deploy
cargo run --release
```

---

## 📎 Anexo: executor_simple.rs (Copy-Paste Ready)

```rust
//! Simple executor que genera URLs de Jupiter para emergencias

use anyhow::Result;
use crate::jupiter::JupiterClient;

pub struct SimpleExecutor {
    jupiter: JupiterClient,
}

impl SimpleExecutor {
    pub fn new() -> Self {
        Self {
            jupiter: JupiterClient::new(),
        }
    }

    pub async fn generate_emergency_sell_url(
        &self,
        token_mint: &str,
        wallet: &str,
        amount: u64,
    ) -> Result<String> {
        println!("🔍 Generando URL de venta de emergencia...");
        
        // 1. Obtener quote
        let sol_mint = "So11111111111111111111111111111111111111112";
        let quote = self.jupiter.get_quote(token_mint, sol_mint, amount, 100).await?;
        
        // 2. Imprimir resumen
        self.jupiter.print_quote_summary(&quote);
        
        // 3. Generar URL
        let url = format!(
            "https://jup.ag/swap/{}-SOL?inAmount={}&slippage=1",
            token_mint,
            amount
        );
        
        println!("\n✅ URL generada:");
        println!("   {}", url);
        
        // 4. Intentar abrir en navegador
        if let Err(e) = webbrowser::open(&url) {
            eprintln!("⚠️  No se pudo abrir navegador: {}", e);
            println!("   Copia el URL manualmente.");
        } else {
            println!("🌐 Navegador abierto automáticamente");
        }
        
        Ok(url)
    }
}
```

Añade a `Cargo.toml`:
```toml
webbrowser = "0.8"
```

---

**¿Quieres que implemente la Ruta Rápida ahora o prefieres explorar otra opción?**
