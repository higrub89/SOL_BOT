# 🚀 SPRINT B: RUSH A AUTONOMÍA TOTAL

**Fecha inicio:** 2026-02-11 16:35  
**Objetivo:** Bot completamente autónomo que compra y vende sin intervención

---

## ✅ **SPRINT 1: RAYDIUM POOL DISCOVERY** (COMPLETADO)

### Implementación:
- ✅ Función `discover_pool_on_chain()` completa
  - Usa `getProgramAccounts` con filtros de memoria
  - Busca pools por base_mint y quote_mint
  - Intenta ambas direcciones (normal e invertida)
  - Parse completo del account de Raydium AMM v4
  - Consulta automática al Serum Market para completar datos
  
- ✅ Función `parse_pool_account()`
  - Extrae todos los campos necesarios del pool
  - Lee correctamente los offsets del AMM v4
  - Obtiene cuentas de Serum (bids, asks, vaults, etc.)
  
- ✅ Función `save_pool_to_cache()`
  - Guarda automáticamente pools descubiertos
  - Evita duplicados en el cache
  - Permite reutilizar pools en futuras operaciones

### Beneficios:
- ✅ **Autonomía:** El bot puede operar con tokens nuevos sin intervención
- ✅ **Performance:** Pools descubiertos se cachean automáticamente
- ✅ **Fallback:** Si un pool no está en cache, lo busca on-chain

### Testing pendiente:
- [ ] Probar descubrimiento con un token graduado reciente
- [ ] Verificar que el cache se actualiza correctamente
- [ ] Medir latencia de descubrimiento (objetivo: <15s)

---

## ⏳ **SPRINT 2: COMPRA AUTOMÁTICA** (SIGUIENTE)

### Tareas:
1. Crear función `auto_buy()` en un nuevo módulo
2. Integrar Raydium + Jupiter con fallback
3. Añadir lógica de validación pre-compra:
   - Balance mínimo SOL
   - Liquidez mínima del pool
   - Slippage razonable
4. Añadir comando `/autobuy <MINT> <SOL>` en Telegram

### Flujo propuesto:
```
COMANDO /autobuy <MINT> 0.025
   ↓
VERIFICAR balance >= 0.025 + gas
   ↓
BUSCAR pool (cache → on-chain)
   ↓
SI pool encontrado → SWAP vía Raydium
SI pool NO encontrado → FALLBACK a Jupiter
   ↓
AÑADIR a targets.json automáticamente
   ↓
INICIAR monitoreo con TSL
```

---

## ⏳ **SPRINT 3: INTEGRACIÓN SCANNER** (PENDIENTE)

### Tareas:
1. Conectar scanner WebSocket → auto-audit
2. Si audit pasa → trigger auto_buy()
3. Configurar filtros en config:
   - Liquidez mínima
   - Holders mínimos
   - Market cap máximo inicial

### Flujo completo:
```
PUMP.FUN GRADUATION (WebSocket)
   ↓
AUTO-AUDIT (2s)
   ↓
SI 🟢 APROBADO → COMPRA AUTOMÁTICA 0.025 SOL
   ↓
AÑADIR A MONITOREO
   ↓
TRAILING SL ACTIVADO
   ↓
VENTA AUTOMÁTICA al +100% o -60%
```

---

## 📊 **Estado Actual del Bot**

### Funcionando:
- ✅ Monitoreo 24/7 de WIF y POPCAT
- ✅ Venta automática con stop-loss
- ✅ Trailing stop-loss
- ✅ Notificaciones Telegram
- ✅ Raydium pool discovery (**NUEVO**)

### En desarrollo:
- ⏳ Compra automática vía Raydium
- ⏳ Scanner + Auto-audit + Auto-buy
- ⏳ State Manager integración completa

---

## 🎯 **Meta Final de Este Sprint**

**Bot autónomo end-to-end:**
1. Detecta token graduado en Pump.fun
2. Audita automáticamente (2s)
3. Compra 0.025 SOL si pasa audit
4. Monitorea con TSL
5. Vende automáticamente al 2X o -60%
6. Repite el ciclo

**Estimado para completar:** 8-10 horas adicionales

---

## 📝 **Próximo Paso Inmediato**

Crear módulo `auto_buyer.rs` con la lógica de compra inteligente que usa Raydium como primera opción y Jupiter como fallback.

**Comando para testear (cuando esté listo):**
```bash
cargo run -- autobuy --mint <NUEVO_TOKEN> --sol 0.025
```

---

**Actualizado:** 2026-02-11 16:45
