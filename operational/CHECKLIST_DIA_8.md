# 🎯 CHECKLIST OPERATIVA - DÍA 8 (2026-02-06)

**Session ID:** 20260206_142338  
**Timestamp:** 14:27 CET  
**Estado de Red:** ⚠️ LATENCIA ELEVADA (176ms / 379ms)

---

## ⚠️ ALERTA DE LATENCIA

```
Health Check:     176.34ms  (Target: <150ms) ❌
Priority Fee API: 379.12ms  (Target: <150ms) ❌
Recomendación:    ESPERAR MEJORA DE RED o OPERAR CON PRECAUCIÓN
```

**Diagnóstico:**  
El terreno NO es óptimo para sniping de alta frecuencia. La latencia está 17% por encima del umbral quirúrgico.

**Opciones:**
1. ✅ **ESPERAR 15-30 min** y volver a verificar latencia
2. ⚠️ **OPERAR CON PRECAUCIÓN:** Solo tokens con liquidez >$50k y narrativa muy fuerte
3. ❌ **EVITAR:** Sniping en los primeros 5 minutos de lanzamiento

---

## 📋 Pre-Flight Checklist

### Sistema
- [x] Session iniciada (20260206_142338) ✅
- [x] RPC configurado (Helius) ✅
- [x] Template de auditoría generado ✅
- [x] Git commit completado (8877192) ✅
- [ ] Latencia <150ms ❌ (PENDIENTE)

### Herramientas Externas
- [ ] Telegram Desktop abierto (@solana_trojanbot)
- [ ] RugCheck.xyz en navegador
- [ ] Dexscreener en navegador
- [ ] Solscan listo para verificación post-trade

### Wallet Status
- [ ] Balance verificado con `wallet_monitor.py`
- [ ] Confirmado que balance >0.1 SOL disponible
- [ ] Main wallet accesible para extracción

---

## 🎯 Estrategia del Día 8

### Objetivo Principal
**Consolidar ganancias con operación de BAJA frecuencia**

### Meta de Ciclos
- Target: **<5 ciclos** (vs. 14 de ayer)
- Ideal: **1 entrada + 1 salida** (2 ciclos totales)

### Criterios de Selección (Más Estrictos por Latencia)
1. **Liquidez:** >$50,000 (duplicado del estándar)
2. **Edad del Token:** >30 minutos (evitar launch sniping)
3. **Narrativa:** AI, Gaming, o Cultura trending
4. **Volume 5m:** Creciente y sostenido
5. **Holders:** >100 (evitar tokens muy nuevos)

### Protocolo de Auditoría (Reforzado)
Dado que la latencia no es óptima, **NO saltarse ningún paso:**

1. **RugCheck Score:** >90 (vs. >85 normal)
2. **Top 10 Holders:** <12% (vs. <15% normal)
3. **LP Burned:** 100% (sin excepciones)
4. **Mint Disabled:** Verificado (sin excepciones)
5. **Dev Wallet:** Identificar y verificar que vendió <30%

### Entry Strategy
- **Tamaño de Posición:** 0.3-0.5 SOL (conservador por latencia)
- **Slippage:** 30% (mayor que usual para compensar latencia)
- **Priority Fee:** 0.01 SOL (duplicado para compensar red lenta)

### Exit Strategy
- **TP1 (2X):** Vender 60% (vs. 50% usual) - Más agresivo
- **TP2 (3X):** Vender 30% - Reducido de 5X
- **TP3 (7X):** Vender 10% - Reducido de 10X
- **Stop Loss:** -25% (vs. -30%) - Más estricto

**Razón del ajuste:** Con latencia alta, es mejor asegurar ganancias antes.

---

## 🚨 Reglas de Abort para Hoy

**ABORTAR inmediatamente si:**
- Latencia sube a >200ms durante la operación
- Token pierde >5% en los primeros 3 minutos post-entrada
- Aparece wallet nueva en top 10 con >10% durante posición abierta
- Priority fee recomendada sube a >0.015 SOL

---

## 📊 Tracking de Fees (Objetivo del Día)

### Meta de Fricción
- **Máximo aceptable:** 0.015 SOL (~$2 USD)
- **Cálculo:**
  - Jito Tip: 0.001 × 2 ciclos = 0.002 SOL
  - Priority Fee: 0.01 × 2 = 0.02 SOL
  - **Total estimado:** 0.022 SOL

⚠️ **Esto está en el límite superior. Monitorear de cerca.**

---

## ✅ Workflow de Operación

### 1. Preparación (AHORA - 15 min)
```bash
# Verificar latencia nuevamente
python3 operational/scripts/helius_engine.py

# Si latencia <150ms, proceder. Si no, ESPERAR.
```

### 2. Búsqueda de Token (15-30 min)
- Ir a Dexscreener: https://dexscreener.com/solana
- Filtrar por liquidez >$50k
- Buscar tokens con 30min - 2h de edad
- Nota: NO perseguir tokens que ya hicieron >3X

### 3. Auditoría (5-10 min)
- Copiar CA
- Pegar en RugCheck: https://rugcheck.xyz
- Completar checklist en: `operational/audits/audit_template_20260206.md`
- **Solo proceder si TODOS los checks pasan**

### 4. Ejecución (1 min)
- Pegar CA en Trojan Bot
- Verificar datos (precio, liquidez, holders)
- Comprar 0.3-0.5 SOL
- **NO multiple entries** - Una sola compra

### 5. Configurar Salidas (2 min)
- `/positions` en Trojan
- Configurar TP1 (2X), TP2 (3X), TP3 (7X)
- Configurar SL (-25%)
- Screenshot de configuración

### 6. Monitoreo (Pasivo)
- Revisar `/positions` cada 15-20 min
- NO hacer trades emocionales
- Dejar que los TP/SL trabajen solos

### 7. Post-Trade (10 min)
- Actualizar `audit_template_20260206.md` con resultado
- Agregar nota en `session_20260206_142338.log`
- Si ganancia >0.1 SOL, extraer a Main Wallet

---

## 🎓 Lecciones de Ayer a Aplicar HOY

1. ✅ **Menos ciclos = Menos fees** → Target: 2 ciclos máximo
2. ✅ **Script detectó fake token** → Confiar en el proceso
3. ✅ **Network deviation = abort** → No forzar trades en malas condiciones
4. ✅ **Jito tips acumulan** → Evitar alta frecuencia

---

## 📞 Contacto de Emergencia

Si algo sale mal:
1. **Kill switch:** Cerrar posición manualmente en Trojan (`/positions` → Sell All)
2. **Verificar en Solscan:** https://solscan.io/
3. **Documentar en logs** para post-mortem

---

## ⏰ Timeline Sugerido

```
14:30 - Re-check latencia
14:35 - Abrir herramientas (Telegram, RugCheck, Dexscreener)
14:40 - Comenzar búsqueda de token
15:00 - Token identificado o seguir buscando
15:10 - Auditoría completada
15:15 - Ejecutar entrada (si aprobado)
15:17 - Configurar salidas
15:20 - 17:00 - Monitoreo pasivo
17:00 - Evaluar cerrar posición si no tocó TP1
```

---

## 🚀 Estado Mental para Operar

**Recordatorio:**
> "El objetivo no es hacer 100 trades. Es hacer el trade correcto 100 veces."

- ✅ Operar desde la lógica, no desde FOMO
- ✅ Si no hay setup perfecto, NO operar
- ✅ Mejor perder una oportunidad que perder capital
- ✅ El mercado estará aquí mañana

---

**Estado:** 🟡 STANDBY - Esperando mejora de latencia  
**Next Action:** Re-verificar latencia en 15 minutos  
**Autorización para operar:** PENDIENTE (latencia debe bajar <150ms)

---

**Última actualización:** 2026-02-06 14:27 CET
