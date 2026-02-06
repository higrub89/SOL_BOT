# 🎯 PROTOCOLO OPERACIONAL - TRADING EN VIVO

**Última Actualización:** 2026-02-05  
**Autor:** Ruben  
**Propósito:** Guía paso a paso para ejecutar operaciones de sniper trading con precisión quirúrgica

---

## 📋 PRE-FLIGHT CHECKLIST (5 minutos)

Antes de comenzar, verifica que TODOS estos elementos estén listos:

### ✅ Software y Herramientas
- [ ] **Terminal abierta** en `/home/ruben/Automatitation/bot_trading`
- [ ] **Telegram Desktop** abierto con @solana_trojanbot
- [ ] **Navegador con 3 pestañas:**
  - [ ] Tab 1: https://rugcheck.xyz
  - [ ] Tab 2: https://dexscreener.com/solana
  - [ ] Tab 3: https://solscan.io
- [ ] **Editor de texto** con tu template de auditoría abierto

### ✅ Configuración Verificada
```bash
# Ejecutar en terminal:
cd /home/ruben/Automatitation/bot_trading
./operational/scripts/trading_session.sh
```

Debe mostrar:
- ✅ Conectividad a Internet: OK
- ✅ RPC configurado
- ✅ Template de auditoría creado
- ✅ Log de sesión inicializado

### ✅ Capital Listo
- [ ] Burner wallet fondeada con **0.5-1 SOL**
- [ ] Wallet principal con fondos de respaldo (NO tocar)
- [ ] Balance verificado en Trojan: `/balance`

### ✅ Mentalidad Correcta
- [ ] Entiendes que puedes perder el 100% de la operación
- [ ] Estás preparado para seguir el protocolo sin emociones
- [ ] Tienes 30-60 minutos sin interrupciones

---

## 🔍 FASE 1: HUNTING (Buscar el Token) - 10-15 min

### Paso 1.1: Filtrar en Dexscreener

1. Ve a: https://dexscreener.com/solana
2. En el buscador, selecciona **"New Pairs"**
3. Aplica estos filtros:
   ```
   Liquidez:     > $10,000
   Edad:         < 1 hora (idealmente < 30 min)
   Volumen 5m:   > $5,000
   Volumen 1h:   > $20,000
   Price Change: +50% a +500% (máximo)
   ```

4. **IMPORTANTE:** Evita tokens que:
   - Ya hicieron 10X+ (llegaste tarde)
   - Tienen menos de 50 holders
   - Tienen liquidez < $5,000
   - Están en descenso continuo (5 velas rojas seguidas)

### Paso 1.2: Análisis Visual Rápido (30 segundos por token)

Para cada candidato, mira el gráfico 5m y busca:
- ✅ **Patrón saludable:** Subidas graduales con consolidaciones
- ✅ **Volumen creciente:** Barras de volumen cada vez más grandes
- ✅ **Holders activos:** Número de holders subiendo
- ❌ **Red flags:** Spike súbito + caída vertical = pump & dump

### Paso 1.3: Copiar Contract Address (CA)

Cuando encuentres un candidato, haz clic en el token y:
1. Copia el **Contract Address (CA)** - ejemplo: `7xKXtg2CW87d9...`
2. Guárdalo temporalmente en tu editor de texto
3. **NO COMPRES TODAVÍA** - Primero auditar

**Meta:** Tener 2-3 CAs candidatas antes de pasar a auditoría

---

## 🔬 FASE 2: AUDIT (Auditoría Quirúrgica) - 3-5 min por token

### Paso 2.1: Abrir Template de Auditoría

```bash
# Abre el template del día:
nano operational/audits/audit_template_$(date +%Y%m%d).md
```

### Paso 2.2: RugCheck - Análisis de Contrato

1. Ve a: https://rugcheck.xyz
2. Pega el **Contract Address**
3. Espera 10-15 segundos a que cargue

#### ✅ Checks Obligatorios:

| Check | Requirement | ¿Cómo verificarlo? |
|-------|-------------|-------------------|
| **LP Burned** | 100% | Debe decir "LP Burned: 100%" o "Liquidity Locked: Burned" |
| **Mint Authority** | Disabled | Debe decir "Mint Authority: Disabled" o "Cannot mint more tokens" |
| **Top 10 Holders** | < 15% cada uno | Ver lista de holders, ninguno debe tener >15% |
| **RugCheck Score** | > 85/100 | Número grande y verde en la parte superior |

#### 🚨 RED FLAGS INMEDIATOS (ABORTAR):
- ❌ LP no burned o locked < 1 año
- ❌ Mint Authority activa (pueden crear tokens infinitos)
- ❌ Un holder con > 20% (probable dev wallet)
- ❌ Score < 70/100
- ❌ Warnings rojos de "High Risk" o "Scam"

### Paso 2.3: Análisis de Distribución en Solscan

1. Ve a: https://solscan.io
2. Pega el CA en el buscador
3. Ve a la pestaña **"Holders"**

#### Verificar:
- **Total Holders:** Mínimo 100, idealmente 200+
- **Top 5 Wallets:** Ninguno con >15% (excepto LP pool)
- **Patrón de compra:** 
  - ✅ Bueno: Muchas wallets pequeñas (0.1-2 SOL)
  - ❌ Malo: Pocas wallets grandes (10+ SOL cada una)

### Paso 2.4: Completar Template de Auditoría

Rellena el template con:
```markdown
## 1. Datos Básicos
- Token CA: 7xKXtg2CW87d9... (tu CA)
- Token Symbol: $EJEMPLO
- Narrativa: Meme de gatos / IA / etc
- Liquidez Inicial: $15,000
- Fecha/Hora: 2026-02-05 11:45

## 2. Telemetría de Seguridad
- [x] LP Burned (100%): ☑ SI
- [x] Mint Authority Disabled: ☑ SI
- [x] Top 10 Holders < 15%: ☑ SI (12.3% max)
- [x] RugCheck Score: 92/100

## 3. Análisis de Distribución
- Total Holders: 287
- Top 5 Wallets (%): 12%, 9%, 8%, 7%, 6%
- Dev Wallet Identificada: ☑ NO (bueno)

## 4. Decisión de Entrada
- [x] APROBADO para entrada: ☑ SI
- Tamaño de Posición: 0.5 SOL
- Precio de Entrada: $ 0.0000123
```

### Paso 2.5: Decision Gate

**SI TODOS LOS CHECKS PASAN:** ✅ Continuar a FASE 3  
**SI FALLA ALGÚN CHECK:** ❌ Descartar token, volver a FASE 1

---

## 💰 FASE 3: ENTRY (Ejecutar la Compra) - 2-3 min

### Paso 3.1: Verificar Condiciones de Red

```bash
# Check de latencia:
python3 operational/scripts/helius_engine.py
```

**Requerimiento:** Latencia < 150ms  
**Si > 150ms:** Condiciones no óptimas, considera esperar o usar priority fee más alto

### Paso 3.2: Compra en Trojan Bot

1. **En Telegram** (@solana_trojanbot):
   ```
   Pega el Contract Address directamente
   ```

2. El bot te mostrará:
   ```
   Token: $EJEMPLO
   Precio: $0.0000123
   Liquidez: $15,000
   Holders: 287
   
   [Buy 0.1 SOL] [Buy 0.5 SOL] [Buy 1 SOL] [Custom]
   ```

3. **Para tu PRIMERA operación:**
   - Selecciona **[Buy 0.5 SOL]** (máximo)
   - O usa **[Custom]** para poner 0.3 SOL si quieres más seguridad

4. **Confirma la transacción:**
   - Revisa que el precio sea correcto
   - Verifica que el slippage sea 25-30%
   - Presiona **[Confirm]**

5. **Espera 10-30 segundos**
   - El bot te notificará: "✅ Buy executed!"
   - Te mostrará tu balance actual

### Paso 3.3: INMEDIATO - Configurar Take Profits

**NO ESPERES NI 1 MINUTO** - Configura inmediatamente:

1. En Trojan, envía: `/positions`
2. Selecciona el token que acabas de comprar
3. Presiona **[Set TP/SL]**

4. **Configurar Take Profits:**
   ```
   TP1 (2X):  Sell 50%   →  Precio: $0.0000246 (el doble)
   TP2 (5X):  Sell 30%   →  Precio: $0.0000615 (5X)
   TP3 (10X): Sell 20%   →  Precio: $0.0001230 (10X)
   
   Stop Loss: Sell 100%  →  Precio: $0.0000086 (-30%)
   ```

5. **Confirma cada uno** y verifica que estén activos

### Paso 3.4: Documentar en Template

Actualiza tu template:
```markdown
## 5. Estrategia de Salida
- [x] TP 1 (2X - 50%): $ 0.0000246 ✅ ACTIVO
- [x] TP 2 (5X - 30%): $ 0.0000615 ✅ ACTIVO
- [x] TP 3 (10X - 20%): $ 0.0001230 ✅ ACTIVO
- [x] Stop Loss (-30% - 100%): $ 0.0000086 ✅ ACTIVO

Hora de Entrada: 11:47
Balance antes: 1.0 SOL
Balance después: 0.5 SOL
Tokens adquiridos: ~40,800,000
```

### Paso 3.5: Screenshot Mental

En este punto deberías tener:
- ✅ Posición abierta en Trojan (`/positions`)
- ✅ 4 órdenes activas (TP1, TP2, TP3, SL)
- ✅ Template de auditoría completado
- ✅ ~0.5 SOL restante en wallet para fees

---

## 👀 FASE 4: MONITOR (Seguimiento) - Tiempo variable

### Paso 4.1: Monitoreo Pasivo (Primeros 15-30 min)

**NO MIRES EL PRECIO CADA 30 SEGUNDOS** - Te vas a estresar.

En su lugar:
1. Configura alertas en Trojan:
   ```
   /settings → Notifications → Price Alerts: ON
   ```

2. Revisa cada 10-15 minutos:
   ```
   /positions
   ```

3. Monitorea holders en Solscan:
   - Si ves que los holders SUBEN = señal positiva
   - Si ves que los holders BAJAN = gente vendiendo, precaución

### Paso 4.2: Escenarios Posibles

#### 🟢 Escenario A: TP1 Ejecutado (2X) ✅
```
Trojan: "✅ TP1 executed! Sold 50% at $0.0000246"
```

**Acción:**
1. ¡CELEBRA! Recuperaste tu inversión inicial 🎉
2. El resto es "house money" (ganancias puras)
3. **Ajusta tu SL**: Muévelo a breakeven (tu precio de entrada)
   ```
   Stop Loss nuevo: $0.0000123 (tu precio original)
   ```
4. Ahora puedes dejar correr el 50% restante sin estrés

#### 🟡 Escenario B: Precio Lateral (±10%)
```
Precio se mueve entre $0.0000110 - $0.0000135
```

**Acción:**
- **PACIENCIA** - No vendas manualmente
- Espera a que toque TP1 o SL
- Revisa holders cada 30 min
- Si holders siguen subiendo = mantén
- Si holders bajan 20%+ = considera vender manual

#### 🔴 Escenario C: Stop Loss Ejecutado (-30%) 💀
```
Trojan: "🛑 Stop Loss executed! Sold 100% at $0.0000086"
```

**Acción:**
1. **ACEPTA LA PÉRDIDA** - Es parte del juego
2. Perdiste 0.15 SOL (~30% de 0.5 SOL)
3. **ANALIZA:**
   - ¿La auditoría tenía algún warning que ignoraste?
   - ¿El mercado general de SOL cayó?
   - ¿Hubo un dump coordinado?
4. **DOCUMENTA** en el template:
   ```markdown
   ## 6. Resultado Final
   - Precio de Salida: $ 0.0000086
   - ROI: -30%
   - Ganancia/Pérdida: -0.15 SOL
   - Lecciones Aprendidas:
     - Ejemplo: "Token tenía muy pocos holders, debí esperar a 200+"
     - Ejemplo: "Mercado general estaba bajando, mal timing"
   ```

### Paso 4.3: Monitoreo de Red Flags DURANTE la operación

Si observas alguno de estos, **VENDE MANUALMENTE**:
- 🚨 Holders caen 30%+ en 10 minutos
- 🚨 Top holder vende > 5% del supply
- 🚨 Liquidez cae > 50%
- 🚨 Precio cae > 50% en 5 minutos sin rebote
- 🚨 RugCheck score cambia a "High Risk"

**Cómo vender manual:**
```
Trojan: /positions → [tu token] → [Sell %] → [Sell 100%] → [Confirm]
```

---

## 🏁 FASE 5: POST-TRADE (Cierre y Análisis) - 5-10 min

### Paso 5.1: Cuando Cierres la Posición

Al salir (por TP, SL, o venta manual):

1. **Calcula tu P&L:**
   ```
   Balance inicial:  1.0 SOL
   Balance final:    ?.?? SOL
   Ganancia/Pérdida: +/- X SOL
   ROI:              +/- X%
   ```

2. **Completa el template:**
   ```markdown
   ## 6. Resultado Final
   - Precio de Salida: $ ____
   - ROI: ___% 
   - Ganancia/Pérdida: ___ SOL
   - Tiempo en posición: __ horas
   - Lecciones Aprendidas:
     - Lo que hice bien:
     - Lo que hice mal:
     - Qué cambiaría:
   ```

3. **Actualiza el log de sesión:**
   ```bash
   echo "
   [$(date)] Trade completed:
   - Token: CA_DEL_TOKEN
   - Entry: \$0.0000123
   - Exit: \$0.0000246
   - ROI: +100%
   - P&L: +0.5 SOL
   " >> operational/logs/session_$(date +%Y%m%d)_*.log
   ```

### Paso 5.2: Transferir Ganancias (Si las hay)

**SI GANASTE** (ROI positivo):
1. En Trojan: `/withdraw`
2. Pega tu **Trading Wallet principal** (NO burner)
3. Retira **SOLO las ganancias**, deja 0.5 SOL en burner para el próximo trade
4. Ejemplo:
   ```
   Ganaste: 0.5 SOL
   Retirar: 0.4 SOL (dejar 0.1 para fees)
   ```

### Paso 5.3: Actualizar Métricas del Proyecto

```bash
nano PROJECT_STATUS.md
```

Actualiza la tabla de métricas:
```markdown
| Métrica | Target | Progreso |
|---------|--------|----------|
| Operaciones Documentadas | 10+ | 1/10 ✅ |
| Primer 2X | 1 | ✅ LOGRADO (o ⏳ Pendiente) |
| Win Rate | >40% | 100% (1/1) (o 0% si perdiste) |
```

---

## 🔄 REPETIR EL CICLO

Una vez completada tu primera operación:
1. **Descansa 30-60 minutos** - No hagas trades emocionales
2. **Analiza tu desempeño**
3. **Ajusta tu estrategia** basado en lecciones aprendidas
4. **Vuelve a FASE 1** cuando estés listo

**Meta de Fase 1:** 
- 10 operaciones documentadas
- Al menos 1 ganador de 2X+
- Win Rate > 40%

---

## ⚠️ REGLAS ABSOLUTAS - REVISIÓN FINAL

Antes de cada trade, repite mentalmente:

1. ✅ **NUNCA** comprar sin auditoría completa
2. ✅ **SIEMPRE** configurar TP/SL inmediatamente
3. ✅ **NUNCA** usar más de 1 SOL por trade (máximo)
4. ✅ **SIEMPRE** vender 50% al 2X
5. ✅ **NUNCA** mover el Stop Loss hacia abajo
6. ✅ **SIEMPRE** documentar cada trade
7. ✅ **NUNCA** operar bajo emociones (FOMO, venganza)
8. ✅ **SIEMPRE** transferir ganancias fuera de burner

---

## 📞 TROUBLESHOOTING RÁPIDO

### "Transaction failed"
- **Causa:** Slippage muy bajo o fees insuficientes
- **Solución:** 
  ```
  /settings → Slippage: 35-40%
  /settings → Priority Fee: 0.01 SOL
  ```

### "Insufficient SOL for rent"
- **Causa:** Balance demasiado bajo
- **Solución:** Mantén siempre 0.05 SOL extra para fees

### "Token not showing in /positions"
- **Causa:** TX pendiente o fallida
- **Solución:** 
  1. Espera 60 segundos
  2. Verifica en Solscan: https://solscan.io/account/TU_WALLET
  3. Si falló, intenta de nuevo

### "Price moved too much during audit"
- **Causa:** Token muy volátil
- **Solución:** 
  - Si subió >50% durante auditoría: **SKIP**, llegaste tarde
  - Si bajó >20%: **SKIP**, posible dump

---

## 🎯 OBJETIVO DE HOY

- [ ] Completar FASE 1-5 con 1 token
- [ ] Documentar la experiencia completa
- [ ] Aprender del resultado (ganancia o pérdida)
- [ ] Actualizar PROJECT_STATUS.md

**Recuerda:** El objetivo de hoy NO es ganar dinero, es **ejecutar el protocolo correctamente**.

Si sigues el proceso, las ganancias vendrán con el tiempo.

---

**¡Buena caza! 🚀**

**Última Actualización:** 2026-02-05 11:45  
**Versión:** 1.0
