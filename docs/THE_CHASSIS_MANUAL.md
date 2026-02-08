# 🏎️ The Chassis - Manual de Operaciones
**Versión:** v0.6.0 (Monitor & Emergency System)
**Estado:** Operacional / Semiautomático

---

## 🏁 1. Inicio Rápido

Para arrancar el motor de trading y monitorear tu posición activa:

```bash
# 1. Navegar al directorio del motor
cd core/the_chassis

# 2. Compilar y Ejecutar en modo Release (Máxima Velocidad)
cargo run --release
```

**Nota:** La primera vez tardará unos minutos en compilar. Las siguientes veces es instantáneo.

---

## ⚙️ 2. Configuración de Misión

Actualmente, los parámetros de la misión se configuran directamente en `src/main.rs`.

**Variables Clave:**
```rust
// Token a monitorear
const ICEBEAR_MINT: &str = "86WM5NBUtRWTHULKrspS1TdzVFAcZ9buXsGRAiFDpump";
const ICEBEAR_ENTRY: f64 = 0.0005687;  // Tu precio medio de compra
const ICEBEAR_INVESTED: f64 = 0.051;   // Cantidad de SOL invertido

// Configuración de Emergencia (src/main.rs - EmergencyConfig)
max_loss_percent: -30.0,    // Stop Loss (e.g., -30%)
min_asset_price: 0.000398,  // Nivel de precio crítico ("Suelo")
```

*Para cambiar de token, edita estas líneas y vuelve a ejecutar `cargo run --release`.*

---

## 📟 3. Interpretación del Dashboard

El bot imprime actualizaciones en tiempo real cada 5 segundos.

### **Indicadores de Estado:**
| Icono | Significado | Acción Recomendada |
| :--- | :--- | :--- |
| 🟢 | **Seguro** (Drawdown 0% a -10%) | Mantener, buscar Take Profit. |
| 🟡 | **Alerta** (Drawdown -10% a -20%) | Vigilar de cerca. Preparar dedo en el gatillo. |
| 🔴 | **Peligro** (Drawdown > -20%) | Zona crítica. Evaluar salida manual. |
| 🚨 | **EMERGENCY** (SL ROTO) | **VENDER INMEDIATAMENTE** en Trojan. |

### **Lectura de Red:**
*   **Latency HTTP:** Mide la congestión general. Si es > 500ms, la red está muy lenta.
*   **WebSocket Stream:** Muestra cambios en tu wallet (compras/ventas) casi al instante (<100ms).

---

## ⚠️ 4. Protocolo de Emergencia

Si ves el mensaje:
```
╔════════════════════════════════════════════════════════════╗
║                  🚨 EMERGENCY ALERT! 🚨                   ║
╚════════════════════════════════════════════════════════════╝
```

**Significa:**
1.  El precio ha roto tu Stop Loss (-30%) o tu Nivel de Precio Mínimo.
2.  El bot **NO VENDE AUTOMÁTICAMENTE** (aún).
3.  **TÚ DEBES:**
    *   Ir a Telegram (Trojan Bot).
    *   Pulsar **"Sell 100%"** inmediatamente.

---

## 🛣️ 5. Roadmap: Siguientes Pasos

Estamos en la fase de transición de "Copiloto" a "Piloto Automático".

### **Fase 1: Configuración Dinámica (Próxima Sesión)**
- Crear `targets.json` para añadir/quitar tokens sin tocar código.
- Soporte para múltiples posiciones simultáneas.

### **Fase 2: El Gatillo (Jito Integration)**
- Implementar `executor.rs`.
- Conectar con Jito Labs Block Engine.
- **Objetivo:** Que el bot ejecute la orden de venta automáticamente cuando salte la alarma 🚨.

### **Fase 3: Velocidad Hipersónica (gRPC Real)**
- Contratar Helius Developer Plan ($49/mo).
- Reemplazar el `scanner.rs` (Dexscreener) por `geyser.rs` (Direct Blockchain Stream).
- **Ventaja:** Ver el precio antes que Dexscreener se actualice.

---

**Ruben's Trading Forge - 2026**
