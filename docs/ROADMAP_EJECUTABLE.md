# 🎯 ROADMAP EJECUTABLE - The Chassis v1.0

**Objetivo:** Completar el ciclo operativo hoy y preparar automatizaciones para la semana

---

## 🚀 FASE 1: HOY - Primera Operación Real (30-45 min)

### ✅ PASO 1: Encontrar y Auditar un Token (5 min)

**Opción A: Usar uno de los ya auditados**
- $GENTLEMEN ya auditado → 🟢 APROBADO
- Mint: `5TATk16oMrt4vsMR8WwQ9AtiPeosdJhXFkp2UhGJpump`

**Opción B: Buscar uno nuevo**
```bash
# Ve a DexScreener Solana trending
# Copia el contract address del que te guste
cd /home/ruben/Automatitation/bot_trading/intelligence/scripts
python3 auto_audit.py <CONTRACT_ADDRESS>

# Si sale 🟢 APROBADO → Continuar
# Si sale 🟡 o 🔴 → Buscar otro
```

---

### ✅ PASO 2: Comprar el Token (2 min)

```bash
cd /home/ruben/Automatitation/bot_trading/intelligence/scripts
python3 chassis_buy.py GENTLEMEN 5TATk16oMrt4vsMR8WwQ9AtiPeosdJhXFkp2UhGJpump 0.05
```

**Lo que hará el script:**
1. Te mostrará el link de Jupiter: `https://jup.ag/swap/SOL-5TAT...`
2. Vas al link y ejecutas la compra
3. Le dices el precio que obtuviste (ej: 0.0003867)
4. El script registra TODO en `targets.json` automáticamente

**Confirmación visual:**
```
✅ GENTLEMEN añadido a targets.json
   • Precio entrada: $0.0003867
   • Stop-Loss: -35%
   • Trailing: ACTIVO
```

---

### ✅ PASO 3: Activar el Monitor de Protección (1 min)

```bash
cd /home/ruben/Automatitation/bot_trading/core/the_chassis
cargo run
```

**Verás esto:**
```
╔════════════════════════════════════════════════════════════╗
║         🏎️  THE CHASSIS - Solana Trading Engine          ║
║           v1.0.0 - Auto-Sell Ready (Production)           ║
╚════════════════════════════════════════════════════════════╝

✅ Configuración cargada:
   • Targets activos: 1
   • Auto-Execute:    ACTIVADO 🔴
   • Intervalo:       5s

🔑 Modo Auto-Execute: Cargando Keypair...
   • Keypair cargado correctamente para 6EJe...

📱 Telegram Notifier: ACTIVADO
```

**✅ CHECK:** Verifica que diga "Keypair cargado correctamente"

---

### ✅ PASO 4: Verificar que Telegram Funciona (30 seg)

1. Abre tu chat de Telegram con el bot
2. Deberías recibir un mensaje de bienvenida
3. Prueba enviando: `/status`
4. El bot debe responderte con el estado actual

---

### ✅ PASO 5: Observar el Monitor Durante 10-30 minutos

**Qué verás:**
```
┌────────────────────────────────────────────────────────┐
│ 🟢 GENTLEMEN Status                                    │
├────────────────────────────────────────────────────────┤
│   Price:    $0.00038510                         │
│   Drawdown: -0.41%                                  │
│   SL Limit: -35.0% (Dist: 34.59%)                    │
└────────────────────────────────────────────────────────┘
```

**Interpretación de los Emojis:**
- 🟢 = Estás seguro (lejos del SL)
- 🟡 = Precaución (a 5-10% del SL)
- 🔴 = Peligro (a menos de 5% del SL)

**Si el precio SUBE mucho (+30% o más):**
- El Trailing Stop se activará
- El SL subirá automáticamente para asegurar ganancias
- Verás en el log: "🎯 Trailing Stop activado! Nuevo SL: -X%"

**Si el precio CAE al -35%:**
- Verás: "🚨 EMERGENCY ALERT! 🚨"
- El bot venderá automáticamente
- Recibirás notificación en Telegram con el signature

---

### ✅ PASO 6: Documentar la Operación (5 min después de cerrar)

Cuando decidas salir (manual o automática):

```bash
cd /home/ruben/Automatitation/bot_trading/operational/logs
nano trade_log_$(date +%Y%m%d).md
```

Anota:
```markdown
# Trade Log - GENTLEMEN

**Fecha:** 2026-02-09
**Token:** GENTLEMEN (5TAT...pump)
**Entrada:** $0.0003867 | 0.05 SOL
**Salida:** $X.XXXXXXX | X.XX SOL
**Resultado:** +X% / -X%
**Duración:** X horas
**Trailing activado:** SÍ/NO
**Notas:** [Lo que aprendiste]
```

---

## 🔥 FASE 2: ESTA SEMANA - Automatizaciones (3-4 horas total)

### 📅 DÍA 1-2: Compra 100% Automática

**Archivo a modificar:** `core/the_chassis/src/main.rs`

**Tarea:**
1. Añadir parsing de argumentos (usar `clap` crate)
2. Detectar modo: `monitor` vs `buy`

**Código a añadir:**

```rust
// Al inicio de main.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "the_chassis")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Compra un token
    Buy {
        /// Contract address del token
        #[arg(short, long)]
        mint: String,
        
        /// Cantidad de SOL
        #[arg(short, long)]
        amount: f64,
    },
    /// Modo monitor (por defecto)
    Monitor,
}
```

**Implementación:**
```rust
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Buy { mint, amount }) => {
            // Ejecutar compra
            execute_buy_command(&mint, amount).await?;
        }
        _ => {
            // Modo monitor (código actual)
            run_monitor().await?;
        }
    }
    
    Ok(())
}
```

**Comando final:**
```bash
cargo run -- buy --mint 5TAT...pump --amount 0.05
```

**Tiempo estimado:** 2 horas (incluyendo testing)

---

### 📅 DÍA 3-4: Comando Telegram /buy

**Archivo a modificar:** `core/the_chassis/src/telegram_commands.rs`

**Tarea:**
Añadir handler para `/buy <MINT> <AMOUNT>`

**Código a añadir:**

```rust
// En telegram_commands.rs

async fn handle_buy_command(&self, args: Vec<&str>) -> String {
    if args.len() < 2 {
        return "❌ Uso: /buy <MINT> <AMOUNT_SOL>".to_string();
    }
    
    let mint = args[0];
    let amount: f64 = match args[1].parse() {
        Ok(a) => a,
        Err(_) => return "❌ Amount inválido".to_string(),
    };
    
    // Quick audit
    let audit_result = quick_audit(mint).await;
    if !audit_result.is_safe() {
        return format!("🔴 Token rechazado:\n{}", audit_result.summary());
    }
    
    // Execute buy
    let buy_result = self.executor.execute_buy(mint, Some(&self.keypair), amount).await;
    
    match buy_result {
        Ok(result) => {
            format!(
                "✅ Compra ejecutada!\n\
                💰 SOL gastado: {:.4}\n\
                💎 Tokens: {:.0}\n\
                📊 Precio: ${:.10}\n\
                🔗 Signature: {}",
                result.sol_spent,
                result.tokens_received,
                result.price_per_token,
                result.signature
            )
        }
        Err(e) => format!("❌ Error: {}", e),
    }
}
```

**Testing:**
```
Tú: /buy 5TAT...pump 0.05
Bot: [Auditando...]
Bot: ✅ Compra ejecutada!
     💰 SOL gastado: 0.0500
     💎 Tokens: 129,238
     ...
```

**Tiempo estimado:** 2 horas

---

### 📅 DÍA 5: Operar con 2-3 Tokens Simultáneamente

**Archivo a modificar:** `core/the_chassis/targets.json`

**Tarea:**
Simplemente añadir más tokens a la lista.

**Ejemplo:**
```json
{
  "targets": [
    {
      "symbol": "GENTLEMEN",
      "mint": "5TAT...",
      "entry_price": 0.0003867,
      "amount_sol": 0.05,
      "stop_loss_percent": -35.0,
      "active": true,
      ...
    },
    {
      "symbol": "TOKEN2",
      "mint": "ABC...",
      "entry_price": 0.0001234,
      "amount_sol": 0.03,
      "stop_loss_percent": -30.0,
      "active": true,
      ...
    },
    {
      "symbol": "TOKEN3",
      "mint": "XYZ...",
      "entry_price": 0.0005678,
      "amount_sol": 0.02,
      "stop_loss_percent": -40.0,
      "active": true,
      ...
    }
  ],
  ...
}
```

**El bot automáticamente:**
- Monitoreará los 3 en paralelo
- Cada uno tendrá su propio SL independiente
- Venderá el que toque su límite primero
- Te notificará de cada uno en Telegram

**Tiempo estimado:** 30 min (es solo añadir entries)

---

## 📊 CHECKLIST DE PROGRESO

### HOY (Antes de terminar la sesión):
- [ ] Token auditado (🟢)
- [ ] Compra ejecutada
- [ ] targets.json actualizado
- [ ] Monitor corriendo
- [ ] Telegram funcionando
- [ ] Log de la operación creado

### ESTA SEMANA:
- [ ] Día 1-2: Compra CLI automática (`cargo run -- buy`)
- [ ] Día 3-4: Comando Telegram `/buy`
- [ ] Día 5: Operar con 3 tokens simultáneos
- [ ] Día 6-7: Revisar métricas y ajustar estrategia

---

## 🎯 MÉTRICAS DE ÉXITO

**Al final de HOY:**
- ✅ 1 operación real completada
- ✅ Bot vigiló al menos 30 minutos
- ✅ Sistema de protección verificado

**Al final de la SEMANA:**
- ✅ 5-10 operaciones documentadas
- ✅ Compra 100% automática funcionando
- ✅ Comando Telegram operativo
- ✅ Portfolio de 2-3 tokens activos
- ✅ Win rate calculado

---

## 🚨 EMERGENCIAS

### Si el bot se cae:
```bash
cd /home/ruben/Automatitation/bot_trading/core/the_chassis
cargo run
```

### Si pierdes conexión y necesitas vender YA:
```
Opción 1: https://jup.ag/swap/<MINT>-SOL
Opción 2: Trojan Bot en Telegram
Opción 3: Phantom wallet directamente
```

### Si Telegram no responde:
```bash
# Verifica las variables de entorno
cat core/the_chassis/.env | grep TELEGRAM

# Debe mostrar BOT_TOKEN y CHAT_ID
```

---

## 📱 COMANDOS TELEGRAM DISPONIBLES

```
/status    - Estado actual de todas las posiciones
/balance   - Balance de la wallet
/stop      - Parar el bot (solo alerta)
/emergency - Vender TODO inmediatamente
```

---

**READY?** Empieza con el PASO 1: Elige tu token y auditalo 🚀
