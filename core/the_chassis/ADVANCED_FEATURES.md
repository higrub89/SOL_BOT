# 🚀 The Chassis v1.0.0 - Advanced Features Guide

## Nuevas Características Implementadas

### 📱 A) Bot Interactivo de Telegram

Tu bot ahora responde a comandos en tiempo real. Puedes controlar todo desde tu móvil.

#### Comandos Disponibles:

*   **`/start`** - Activa el bot y muestra la lista de comandos
*   **`/status`** - Ver el estado actual de TODOS tus tokens
    *   Precio actual
    *   Drawdown (pérdida/ganancia desde entrada)
    *   Valor actual en SOL
    *   Estado visual (🟢🟡🔴)

*   **`/balance`** - Consultar tu balance de SOL en la wallet

*   **`/targets`** - Lista completa de tokens que estás monitoreando
    *   Stop-Loss configurado
    *   Inversión en SOL
    *   Estado (Activo/Pausado)

*   **`/help`** - Ayuda de todos los comandos

#### ¿Cómo Usar?

1.  Abre Telegram y busca a tu bot (`@solruben_bot`)
2.  Escribe `/status` y en 2 segundos recibes el reporte completo
3.  No necesitas estar en la computadora, funciona desde cualquier lugar

---

### 🎯 B) Trailing Stop-Loss Inteligente

**¿Qué es?** Un stop-loss que "sube" automáticamente when el precio sube, protegiendo tus ganancias.

#### Ejemplo Real:

1.  **Compras ICEBEAR** a $0.001 con Stop-Loss al -50% (precio de venta: $0.0005)
2.  **El precio sube a $0.002** (+100% de ganancia)
3.  **El Trailing SL se activa** (configurado para activarse a +50%)
4.  **Nuevo Stop-Loss automático**: En lugar de $0.0005, ahora es ~$0.0014
    *   ¿Por qué? Porque el trailing permite caer 30% desde el pico ($0.002)
5.  **Si el precio sigue subiendo a $0.003**, el SL sube a ~$0.0021
6.  **Si el precio cae a $0.0020**, ¡VENTA AUTOMÁTICA!
    *   Resultado: En lugar de perder -50%, ahora ganas +100% 🎉

#### Configuración en `targets.json`:

```json
{
  "symbol": "ICEBEAR",
  "trailing_enabled": true,
  "trailing_distance_percent": 30.0,           // Permite caer 30% desde el pico
  "trailing_activation_threshold": 50.0        // Se activa cuando ganas +50%
}
```

#### Parámetros Explicados:

*   **`trailing_enabled`**: `true` para activar, `false` para usar SL fijo
*   **`trailing_distance_percent`**: Cuánto puede caer desde el pico antes de vender
    *   30% = conservador (protege ganancias rápido)
    *   50% = agresivo (deja espacio para volatilidad)
*   **`trailing_activation_threshold`**: A partir de qué ganancia se activa
    *   50% = se activa cuando ganas +50%
    *   100% = se activa cuando duplicas tu inversión

---

### 🐋 C) Alertas de Liquidez y Detector de Ballenas

**¿Qué detecta?**

1.  **Caídas dramáticas de liquidez** (posible rug pull)
2.  **Spikes sospechosos de volumen** (ballenas entrando/saliendo)
3.  **Señales de Rug Pull** (caída de precio + caída de liquidez simultánea)

#### Alertas que Recibirás:

##### 1. Alerta de Liquidez:
```
⚠️ ALERTA DE LIQUIDEZ - ICEBEAR

💧 Caída de liquidez: -35.2%
└─ Antes: $150,000
└─ Ahora: $97,000

🔍 Esto puede indicar ventas grandes o retiro de LP.
```

##### 2. Volumen Anormal:
```
📊 VOLUMEN ANORMAL - ICEBEAR

🚨 Spike de volumen: 8.5x del promedio
└─ Actual 24h: $850,000
└─ Promedio: $100,000

⚠️ Puede indicar actividad de ballenas o dump inminente.
```

##### 3. Advertencia de Rug Pull:
```
🚨🚨 ADVERTENCIA DE RUG PULL - ICEBEAR 🚨🚨

❌ Precio: -42.1%
❌ Liquidez: -58.3%

⚡ ACCIÓN INMEDIATA RECOMENDADA
Considera salir de la posición ahora.
```

#### ¿Cómo Funciona?

El bot mantiene un historial de los últimos 10 "snapshots" de:
*   Liquidez en USD
*   Volumen 24h
*   Precio

Cada 5 segundos (o el intervalo que configures), compara los datos nuevos con el historial y detecta:
*   Cambios >20% en liquidez = Alerta
*   Volumen >5x del promedio = Alerta
*   Precio cayendo + Liquidez cayendo = Rug Pull Warning

---

## 🎛️ Configuración Completa de `targets.json`

Aquí tienes un ejemplo completo con TODAS las features activadas:

```json
{
  "targets": [
    {
      "symbol": "ICEBEAR",
      "mint": "86WM5NBUtRWTHULKrspS1TdzVFAcZ9buXsGRAiFDpump",
      "entry_price": 0.0005687,
      "amount_sol": 0.051,
      "stop_loss_percent": -50.0,
      "panic_sell_price": 0.00028,
      "active": true,
      
      // Trailing Stop-Loss
      "trailing_enabled": true,
      "trailing_distance_percent": 30.0,
      "trailing_activation_threshold": 50.0
    }
  ],
  "global_settings": {
    "min_sol_balance": 0.01,
    "jito_tip_lamports": 50000,
    "auto_execute": false,
    "monitor_interval_sec": 5
  }
}
```

---

## 🚀 Cómo Arrancar el Bot con Todo Activado

```bash
cd /home/ruben/Automatitation/bot_trading/core/the_chassis
cargo run --release
```

---

## 💡 Tips de Uso

### Para Scalpers (Ganancias Rápidas):
```json
"trailing_enabled": true,
"trailing_distance_percent": 20.0,    // Toma profit rápido
"trailing_activation_threshold": 30.0  // Activa temprano
```

### Para Holders (Máximas Ganancias):
```json
"trailing_enabled": true,
"trailing_distance_percent": 50.0,    // Deja espacio
"trailing_activation_threshold": 100.0 // Solo si 2x
```

### Para Day Traders:
```json
"trailing_enabled": false,             // SL fijo
"stop_loss_percent": -20.0            // Tight stop
```

---

## 📊 Ejemplo de Sesión Real

```
Tu (desde Telegram): /status

Bot: 
📊 STATUS DE POSICIONES

🟢 ICEBEAR
└─ Precio: $0.00085123
└─ Entrada: $0.00056870
└─ Drawdown: +49.72%
└─ Valor: 0.0753 SOL

🎯 Trailing SL: ACTIVO
└─ SL Actual: +20.5% (ajustado desde -50%)
└─ Protegiendo ganancias ✅

------------------------

Tu (más tarde): /balance

Bot:
💰 BALANCE DE WALLET

SOL: 0.1484
USD (aprox): $14.84
```

---

## 🎉 Resumen de Superpoderes

| Feature | Antes | Ahora |
|---------|-------|-------|
| **Control** | Solo desde terminal | Desde cualquier lugar con Telegram |
| **Stop-Loss** | Fijo (-50%) | Dinámico (sube con el precio) |
| **Protección** | Reactiva (solo precio) | Proactiva (liquidez + volumen) |
| **Visibilidad** | Consola | Notificaciones móviles 24/7 |

---

**¿Dudas?** Escríbele `/help` a tu bot o revisa este documento.

**Versión**: 1.0.0  
**Fecha**: 2026-02-08  
**Desarrollado por**: Ruben  
