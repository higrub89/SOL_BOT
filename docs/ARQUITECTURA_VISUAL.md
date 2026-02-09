# 🚀 THE CHASSIS v1.0 - Arquitectura Visual

```
┌─────────────────────────────────────────────────────────────────┐
│                    🎯 FLUJO OPERATIVO COMPLETO                  │
└─────────────────────────────────────────────────────────────────┘

1️⃣  DETECCIÓN
    📱 DexScreener / Twitter / Telegram
         │
         ▼
2️⃣  AUDITORÍA (2 segundos)
    🧠 auto_audit.py
         ├─► RugCheck API (Score, LP, Authorities)
         ├─► DexScreener API (Liquidez, Volumen)
         └─► 📄 Reporte Markdown
              ├─► 🟢 APROBADO → Continuar
              ├─► 🟡 RIESGO MEDIO → Revisar manual
              └─► 🔴 PELIGRO → Descartar
         │
         ▼
3️⃣  COMPRA (Semi-automática)
    💰 chassis_buy.py
         ├─► Genera URL Jupiter
         ├─► Usuario confirma compra
         ├─► Registra en targets.json
         │    ├─ Entry price
         │    ├─ Stop-Loss (-35%)
         │    ├─ Trailing Stop (activado)
         │    └─ Amount (0.05 SOL)
         └─► ✅ Listo para protección
         │
         ▼
4️⃣  PROTECCIÓN (Automática 24/7)
    🏎️  The Chassis (Rust)
         ├─► 📡 Escaneo de precio (cada 5s)
         ├─► 📊 Cálculo de Drawdown
         ├─► 🛡️  Trailing Stop-Loss Monitor
         ├─► 📱 Notificaciones Telegram
         └─► ⚡ Auto-Sell (si toca SL)
              └─► 🪙 Jupiter Swap
                   └─► 💸 SOL de vuelta en wallet


┌─────────────────────────────────────────────────────────────────┐
│                   🏗️  ARQUITECTURA DEL SISTEMA                  │
└─────────────────────────────────────────────────────────────────┘

📁 bot_trading/
│
├─ 🧠 intelligence/          # NUEVO - Decisiones inteligentes
│  ├─ scripts/
│  │  ├─ auto_audit.py      # Auditoría 30x más rápida
│  │  └─ chassis_buy.py     # Orquestador de compra
│  ├─ datasets/             # (Futuro: datos históricos)
│  └─ models/               # (Futuro: ML models)
│
├─ 🏎️  core/                 # Motor de ejecución (Rust)
│  └─ the_chassis/
│     ├─ src/
│     │  ├─ executor_v2.rs   # ✅ execute_buy + execute_sell
│     │  ├─ jupiter.rs       # ✅ BuyResult + SwapResult
│     │  ├─ trailing_sl.rs   # Trailing Stop Loss
│     │  ├─ telegram.rs      # Notificaciones
│     │  └─ main.rs          # Orquestador principal
│     ├─ proto/
│     │  └─ chassis.proto    # ✅ Contrato gRPC (Fase 2)
│     └─ targets.json        # Configuración dinámica
│
├─ 📋 operational/           # Día a día
│  ├─ scripts/
│  │  ├─ audit_sniper.py    # Auditoría rápida (legacy)
│  │  └─ wallet_monitor.py  # Monitor de balance
│  ├─ audits/               # ✅ Reportes de tokens
│  └─ logs/
│     └─ simulated_trades.csv # ✅ Paper trading log
│
└─ 📚 docs/
   ├─ FLUJO_OPERATIVO.md           # ✅ Manual de uso
   ├─ RESUMEN_SESION_2026-02-09.md # ✅ Resumen de hoy
   ├─ THE_CHASSIS_ARCHITECTURE.md  # Arquitectura técnica
   └─ EMERGENCY_SYSTEM.md          # Sistema de emergencia


┌─────────────────────────────────────────────────────────────────┐
│                    ⚡ CAPACIDADES DEL SISTEMA                   │
└─────────────────────────────────────────────────────────────────┘

✅ Auditoría Instantánea
   • RugCheck + DexScreener en 2s
   • Veredicto automático (🟢🟡🔴)
   • Reportes Markdown guardados

✅ Compra Semi-Automática
   • Registro automático en targets.json
   • Configuración de SL/Trailing
   • Link directo a Jupiter

✅ Protección 24/7
   • Monitoreo de precio en tiempo real
   • Stop-Loss dinámico (Trailing)
   • Ejecución automática de ventas
   • Notificaciones Telegram

✅ Paper Trading Realista
   • Quotes reales de Jupiter
   • Registro de simulaciones
   • Backtesting preparado

🚧 Próximamente
   • Compra 100% automática
   • Comando Telegram /buy
   • gRPC Server (Python ↔ Rust)
   • Sniper Mode (bloque 0)


┌─────────────────────────────────────────────────────────────────┐
│                     📊 MÉTRICAS DE MEJORA                       │
└─────────────────────────────────────────────────────────────────┘

Proceso                    Antes        Ahora        Mejora
─────────────────────────────────────────────────────────────────
Auditoría                  60s manual   2s auto      30x ⚡
Registro targets.json      Manual       Auto         100% 🎯
Precisión simulación       Fake data    Real quotes  ∞ 📈
Gestión de riesgo          Manual       Auto-SL      24/7 🛡️
Tiempo respuesta SL        Humano (>1m) Bot (<1s)    60x+ ⚡


┌─────────────────────────────────────────────────────────────────┐
│                    🔐 SEGURIDAD IMPLEMENTADA                    │
└─────────────────────────────────────────────────────────────────┘

[Multi-Layer Defense]

1. Pre-Entrada
   └─► Auditoría obligatoria (auto_audit.py)

2. Durante Operación  
   ├─► Stop-Loss activo (-35%)
   ├─► Trailing Stop (asegura ganancias)
   ├─► Balance mínimo (0.01 SOL)
   └─► Telegram alerts en tiempo real

3. Post-Operación
   └─► Logs permanentes de todas las acciones

4. Infraestructura
   ├─► .env protegido (.gitignore)
   ├─► Keypair cifrada en memoria
   └─► API privada (Helius RPC)


┌─────────────────────────────────────────────────────────────────┐
│                   🎮 COMANDOS QUICK-START                       │
└─────────────────────────────────────────────────────────────────┘

# Auditar un token
cd intelligence/scripts
python3 auto_audit.py <MINT_ADDRESS>

# Comprar (registra automáticamente)
python3 chassis_buy.py <SYMBOL> <MINT> <AMOUNT_SOL>

# Activar protección
cd ../../core/the_chassis
cargo run

# Ver todo funcionando ✨


┌─────────────────────────────────────────────────────────────────┐
│                      🎯 ESTADO ACTUAL                           │
└─────────────────────────────────────────────────────────────────┘

Token en Monitor:  $GENTLEMEN
Precio Entrada:    $0.0003867
Inversión:         0.05 SOL
Stop-Loss:         -35%
Trailing:          ✅ Activo
Auto-Execute:      ✅ ON
Telegram:          ✅ Conectado
Keypair:           ✅ Cargada

🟢 Sistema 100% operativo y listo para trading real
```
