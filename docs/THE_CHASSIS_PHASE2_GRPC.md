# 🏎️ THE CHASSIS - Fase 2: gRPC Integration

## 📋 Objetivo
Implementar streaming en tiempo real desde Yellowstone Geyser para reducir la latencia de detección de trades de **~150ms (HTTP)** a **<50ms (gRPC)**.

---

## 🔧 Implementación Actual (v0.2.0)

### ✅ Completado
- [x] Estructura base del módulo `geyser.rs`
- [x] Cliente simulado con métodos de conexión
- [x] Benchmark comparativo HTTP vs gRPC
- [x] Compilación exitosa en Rust

### 🔄 En Progreso  
- [ ] Integración con Yellowstone Geyser **real** (requiere endpoint)
- [ ] Proto definitions para gRPC
- [ ] Account subscription a pools de liquidez
- [ ] Parser de Account Updates

### 🚀 Próximos Pasos
- [ ] WebSocket fallback si gRPC no disponible
- [ ] Integración con Jito Bundles para ejecución
- [ ] Dashboard en tiempo real (opcional)

---

## 📊 Resultados Esperados

| Métrica | HTTP JSON-RPC | gRPC Streaming | Mejora |
| :--- | :--- | :--- | :--- |
| **Latencia Promedio** | ~120ms | ~30ms | **75%** |
| **Estabilidad** | Variable (50-300ms) | Consistente (20-40ms) | **Alta** |
| **Conexión** | Request/Response | Persistent Stream | **Mejor** |

---

## 🛠️ Requisitos Técnicos

### Yellowstone Geyser Endpoints
Para producción, necesitaremos acceso a uno de estos:
- **Helius Premium** (Requiere upgrade de plan)
- **Triton RPC** (Alternativa)
- **Self-hosted Geyser** (Infraestructura propia)

### Dependencias Rust
```toml
tonic = "0.12"           # gRPC framework
prost = "0.13"           # Protobuf
tokio = { version = "1", features = ["full"] }
```

---

## 💡 Ventaja Competitiva

Con gRPC implementado, seremos capaces de:
1. **Ver compras antes que Dexscreener** (100-200ms advantage)
2. **Detectar rug pulls en tiempo real** (monitoring de LP removals)
3. **Ejecutar trades con latencia sub-50ms** (critical para sniping)

---

**Status:** 🟡 Simulado (Esperando endpoint de producción)  
**Última Actualización:** 2026-02-08 02:05 CET  
**Versión:** v0.2.0
