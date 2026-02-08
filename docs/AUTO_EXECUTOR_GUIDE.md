# ⚡ The Chassis v0.7.0 - Auto-Executor Guide

## 🚨 MODO DE OPERACIÓN: Safe vs Armed

The Chassis ahora tiene dos modos de operación que defines en `src/main.rs`:

### **🟡 SAFE MODE (Recomendado para empezar)**
```rust
const AUTO_EXECUTE: bool = false;
```

**Comportamiento:**
- ✅ Monitorea precio en tiempo real
- ✅ Calcula drawdown automáticamente
- ✅ **ALERTA** cuando se rompe el Stop Loss
- ❌ NO ejecuta ventas automáticamente
- **Requiere:** Acción manual en Trojan

**Usa este modo para:**
- Ganar confianza en el sistema
- Verificar que las alertas sean precisas
- Familiarizarte con la dinámica del bot

---

### **🔴 ARMED MODE (Solo cuando confíes 100%)**
```rust
const AUTO_EXECUTE: bool = true;
```

**Comportamiento:**
- ✅ Monitorea precio en tiempo real
- ✅ Calcula drawdown automáticamente
- ✅ **VENDE AUTOMÁTICAMENTE** cuando se rompe el SL
- ✅ Usa Jito Bundles para máxima velocidad
- ⚠️  **PELIGRO:** No hay "¿Estás seguro?"

**Usa este modo para:**
- Protección 24/7 (dormir tranquilo)
- Trading de alta frecuencia
- Cuando estés fuera del ordenador

---

## 📋 Checklist Pre-Activación (Armed Mode)

**ANTES de cambiar `AUTO_EXECUTE` a `true`, verifica:**

1. **✅ Funding Suficiente:**
   - Tienes al menos 0.01 SOL extra para:
     - Jito Tip (~0.00001 SOL)
     - Network Fees (~0.000005 SOL)

2. **✅ Configuración Correcta:**
   - `ICEBEAR_ENTRY` es tu precio real de entrada
   - `ICEBEAR_INVESTED` es tu inversión real en SOL
   - `max_loss_percent` es tu tolerancia de pérdida (e.g., -30%)

3. **✅ Testing en Safe Mode:**
   - Has visto al menos 2-3 alertas correctas en Safe Mode
   - Confías en que el bot detecta los niveles correctamente

4. **✅ Conexión Estable:**
   - Tu latencia HTTP es < 500ms consistentemente
   - No estás en una red WiFi pública

5. **✅ Backup Plan:**
   - Tienes Trojan abierto en el móvil por si algo falla

---

## 🔧 Configuración Avanzada

### **Ajustar el Jito Tip:**
En `src/executor.rs`, línea ~27:
```rust
jito_tip_lamports: 10_000,  // 0.00001 SOL
```

**Recomendaciones:**
- **Mercado Calmado:** 10,000 lamports (0.00001 SOL)
- **Alta Volatilidad:** 50,000 lamports (0.00005 SOL)
- **Emergencia Crítica:** 100,000 lamports (0.0001 SOL)

**Nota:** Más tip = mayor prioridad, pero también mayor costo por operación.

---

### **Cambiar el Intervalo de Monitoreo:**
En `src/main.rs`, línea ~168:
```rust
5, // Check cada 5 segundos
```

**Recomendaciones:**
- **Tokens Estables:** 10-15 segundos
- **Memecoins Volátiles:** 3-5 segundos (actual)
- **Ultra-HFT:** 1 segundo (requiere Helius Premium)

---

## 🎯 Roadmap: De Simulación a Producción

### **Estado Actual (v0.7.0):**
- [x] Estructura del executor
- [x] Detección de emergencias
- [x] Integración con alertas
- [x] Modo Dry-Run (simulación)
- [ ] **Falta:** Construcción real de transacciones

### **Siguiente Paso (v0.8.0):**
- [ ] Integrar Jupiter Aggregator API
- [ ] Construir instrucción de Swap (Token → SOL)
- [ ] Crear Jito Bundle real
- [ ] Testing en Devnet
- [ ] **PRODUCCIÓN:** Deployment en Mainnet

---

## ⚠️ Advertencias Críticas

1. **🚫 NO actives Armed Mode sin haber testeado en Safe Mode primero.**
2. **🚫 NO uses Armed Mode si tu conexión es inestable.**
3. **🚫 NO dejes el bot corriendo sin supervisión hasta v0.8.0.**
4. **✅ SÍ mantén Trojan como backup manual.**

---

**The Chassis está casi listo para volar solo. Pero como todo sistema autónomo, requiere confianza ganada con testing exhaustivo.**

🏎️💨 *Ruben's Trading Forge - Feb 2026*
