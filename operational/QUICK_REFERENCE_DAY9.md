# ⚡ REFERENCIA RÁPIDA - DÍA 9 (MODO SNIPER ACTIVO)

**Session ID:** 20260208_005459  
**Timestamp:** 2026-02-08 00:54 CET  
**Balance:** 0.162 SOL (~$14.20)  
**Latencia:** 🟢 109ms (ÓPTIMO)

---

## 🎯 PARÁMETROS DE OPERACIÓN

### Capital Disponible
```
Balance Total:      0.162 SOL
- Reserve (fees):   -0.020 SOL
= Capital Trading:   0.140 SOL (~$12.30)

Tamaño por Trade:   0.06-0.08 SOL
Trades Posibles:    1-2 (máximo)
```

### 🚨 REGLA CRÍTICA DEL DÍA
```
╔═══════════════════════════════════════════════════════════╗
║  MÁXIMO 1-2 CICLOS POR OPERACIÓN                         ║
║  (Lección del Día 7: 14 ciclos = 0.127 SOL en fricción)  ║
╚═══════════════════════════════════════════════════════════╝
```

---

## 📋 CHECKLIST DE ENTRADA (USA ESTO)

### ANTES de Comprar CUALQUIER Token:

#### 1️⃣ Auditoría Básica (30 segundos)
```bash
# Abrir en RugCheck
https://rugcheck.xyz/[TOKEN_ADDRESS]

VERIFICAR:
□ RugCheck Score: >85/100
□ LP Burned: 100%
□ Mint Authority: Disabled
□ Top 10 Holders: <15% cada uno
```

#### 2️⃣ Análisis de Narrativa (60 segundos)
```bash
# En Dexscreener + Twitter/X
□ Liquidez: >$20k
□ Volumen 24h: >$50k
□ Trending en X: >100 menciones/hora
□ No es token "viejo" (creado hace <24hrs)
```

#### 3️⃣ Confirmación de Ejecución
```bash
Si TODO lo anterior = ✅
ENTONCES:
  1. Copiar Contract Address
  2. Pegar en Trojan Bot
  3. Comprar 0.06-0.08 SOL
  4. INMEDIATAMENTE setear alertas de precio
```

---

## 💰 ESTRATEGIA DE SALIDA

### Operación de 1 Ciclo (PREFERIDA)
```
Entrada: 0.08 SOL

TP1 (2X):   Vender 50% = 0.08 SOL (Break-even)
            → Dejar 50% = 0.08 SOL equivalente en tokens

TP2 (5X):   Vender 25% del restante
            → Ganancia: ~0.15 SOL

Moon Bag:   Dejar 25% para 10X-100X

Stop Loss:  Si cae -30% sin tocar TP1 → VENDER TODO
```

### Cálculo de Ganancia Real (con fricción)
```
Ganancia Bruta al 5X:    +0.32 SOL
- Jito Tip (entrada):     -0.0075 SOL
- Jito Tip (TP1):         -0.0075 SOL
- Jito Tip (TP2):         -0.0075 SOL
- Priority Fees (x3):     -0.006 SOL
= Ganancia Neta:          +0.29 SOL (~$25.50)

ROI Real: +179% (vs +400% bruto)
```

**Por esto necesitamos targets ALTOS (5X-10X).**

---

## 🔴 SEÑALES DE ALERTA - ABORTAR

### Abortar INMEDIATAMENTE si:
- ❌ RugCheck score <85
- ❌ Wallet con >20% de supply
- ❌ LP no burned
- ❌ Latencia sube >200ms (re-check con `helius_engine.py`)
- ❌ Liquidez <$10k
- ❌ "Vibes" malos (confía en tu instinto)

---

## 🛠️ COMANDOS RÁPIDOS

### Re-verificar Balance
```bash
python3 operational/scripts/wallet_monitor.py HF2UG1JNMuh7vhT4Bt1WehVhvnPzVLLTBUJD4bKY7dQv
```

### Re-verificar Latencia
```bash
python3 operational/scripts/helius_engine.py
```

### Completar Audit Template
```bash
nano operational/audits/audit_template_20260208.md
```

### Ver Log de Sesión
```bash
tail -f operational/logs/session_20260208_005459.log
```

---

## 🎓 RECUERDA

### Del Post-Mortem Día 7:
1. **"El objetivo no es hacer 100 trades. Es hacer el trade correcto 100 veces."**
2. **Alta frecuencia = Alta fricción** (14 ciclos = pérdida del 91% de ganancia)
3. **Jito Bundles > Jito Tips** (próximo upgrade en The Chassis)

### Filosofía de Hoy:
```
1 trade perfecto con 5X = Mejor que 5 trades mediocres con 2X
```

---

## 📊 TRACKING DE OPERACIONES

### Formato de Nota Rápida
```
Token: $SYMBOL
CA: [contract_address]
Entrada: 0.0X SOL @ $X.XX
Timestamp: HH:MM
RugCheck: XX/100
Narrativa: [1 línea]
---
TP1 (2X): [ ] @ $X.XX | HH:MM
TP2 (5X): [ ] @ $X.XX | HH:MM
SL (-30%): [ ] @ $X.XX | HH:MM
```

---

## 🔗 LINKS ACTIVOS

- **Wallet:** https://solscan.io/account/HF2UG1JNMuh7vhT4Bt1WehVhvnPzVLLTBUJD4bKY7dQv
- **RugCheck:** https://rugcheck.xyz
- **Dexscreener:** https://dexscreener.com/solana
- **Trojan Bot:** https://t.me/solana_trojanbot

---

**Estado:** 🟢 MODO SNIPER ACTIVO  
**Capital:** 0.162 SOL  
**Latencia:** 109ms (ÓPTIMO)  
**Target Hoy:** 1 operación perfecta con 5X+  

**¡Buena caza! 🎯**
