# 📘 THE BLUE BOOK - Documentación Institucional

**Proyecto:** The Chassis - Solana Trading Engine  
**Versión:** 2.0.0 (Framework Institucional)  
**Estado:** En Desarrollo Activo  
**Fecha de Inicio:** 2026-02-09

---

## 🎯 Qué es The Blue Book

The Blue Book es la **documentación técnica de grado institucional** de The Chassis. Inspirado en estándares de ingeniería de sectores críticos (automoción, aeroespacial, defensa), este conjunto de documentos garantiza que cualquier ingeniero pueda:

1. **Comprender** la arquitectura del sistema en profundidad
2. **Operar** el bot con confianza
3. **Debuggear** problemas en minutos, no horas
4. **Extender** el sistema sin romper abstracciones
5. **Auditar** cada decisión técnica tomada

---

## 📚 Documentos Disponibles

### 1. [TELEMETRY_MANUAL.md](./TELEMETRY_MANUAL.md)
**Manual de Telemetría - Los Logs de "Hiperlujo"**

- Niveles de log (TRACE, DEBUG, INFO, WARN, ERROR)
- Formato estándar de logs estructurados
- Módulos del sistema (EXECUTOR, AUDIT, EMERGENCY, etc.)
- Ejemplos de sesiones completas
- Macros de conveniencia
- Estrategias de monitoreo en tiempo real

**Cuándo leer:** Antes de analizar logs o implementar nuevos módulos.

---

### 2. [ARCHITECTURE_BLUEPRINT.md](./ARCHITECTURE_BLUEPRINT.md)
**Blueprint Arquitectónico - El Mapa del Sistema**

- Diagrama de capas (Operator → Orchestration → Execution → Intelligence → Data)
- Flujo completo de un trade (Detection → Audit → Execution → Monitoring)
- Componentes técnicos clave:
  - Executor Trait (polimorfismo)
  - gRPC Bridge (Rust ↔ Python)
  - Observability System
  - Persistencia con SQLite
- Principios de diseño (Soberanía, Resiliencia, Observabilidad)
- Roadmap de evolución (4 fases)

**Cuándo leer:** Al incorporarte al proyecto o diseñar nuevas features.

---

### 3. [SECURITY_VAULT.md](./SECURITY_VAULT.md) *(Próximamente)*
**Bóveda de Seguridad - Manejo de Secretos**

- Inyección de variables de entorno
- Uso de `secrecy` y `zeroize` en Rust
- Eliminación de archivos `.env` en producción
- Protocolos de rotación de claves
- Auditoría de accesos

**Cuándo leer:** Antes de deployar en producción.

---

### 4. [DEPLOYMENT_GUIDE.md](./DEPLOYMENT_GUIDE.md) *(Próximamente)*
**Guía de Despliegue - Del Dev a Producción**

- Setup de servidor VPS (recomendaciones)
- Configuración de systemd para auto-restart
- Monitoreo con Prometheus + Grafana
- Integración con Telegram para alertas críticas
- Rollback procedures

**Cuándo leer:** Al preparar el bot para trading real 24/7.

---

### 5. [RAYDIUM_DEEP_DIVE.md](./RAYDIUM_DEEP_DIVE.md) *(Próximamente)*
**Inmersión Profunda en Raydium - El Motor de Velocidad**

- Anatomía de un pool AMM v4
- Layout binario de cuentas
- Construcción de instrucciones swap
- Cálculo de slippage óptimo
- Pool discovery strategies

**Cuándo leer:** Al implementar Raydium Executor (Sprint 1-4).

---

## 🏗️ Filosofía de la Documentación

> "En el sector de alta gama, la documentación es tan importante como el código. Si no está documentado, no existe."

### Principios

1. **Precisión Quirúrgica:** Cero ambigüedades. Cada término técnico definido.
2. **Ejemplos Reales:** Código ejecutable, no pseudocódigo.
3. **Diagramas ASCII:** Visualización rápida sin dependencias externas.
4. **Versionado:** Cada documento indica su versión y fecha de actualización.
5. **Trade-offs Explícitos:** Documentamos por qué elegimos X sobre Y.

---

## 📊 Estado de Completitud

| Documento | Estado | Prioridad | ETA |
|-----------|--------|-----------|-----|
| TELEMETRY_MANUAL.md | ✅ Completo | Alta | N/A |
| ARCHITECTURE_BLUEPRINT.md | ✅ Completo | Alta | N/A |
| SECURITY_VAULT.md | 🚧 Pendiente | Alta | 2026-02-10 |
| DEPLOYMENT_GUIDE.md | 🚧 Pendiente | Media | 2026-02-12 |
| RAYDIUM_DEEP_DIVE.md | 🚧 Pendiente | Media | Sprint 2 |

---

## 🎓 Cómo Usar The Blue Book

### Para Nuevos Colaboradores
1. Leer `ARCHITECTURE_BLUEPRINT.md` (30 min)
2. Revisar `TELEMETRY_MANUAL.md` (20 min)
3. Explorar código con referencia constante a los diagramas

### Para Debugging
1. Identificar módulo problemático
2. Consultar formato de log en `TELEMETRY_MANUAL.md`
3. Grep los logs con patrones correctos
4. Comparar flujo esperado con el diagrama en `ARCHITECTURE_BLUEPRINT.md`

### Para Nuevas Features
1. Verificar principios de diseño en `ARCHITECTURE_BLUEPRINT.md`
2. Diseñar respetando abstracciones existentes (Executor Trait, etc.)
3. Documentar logs según `TELEMETRY_MANUAL.md`
4. Actualizar Blue Book con cambios arquitectónicos

---

## 🚀 Próximos Pasos

### Documentación
- [ ] Completar `SECURITY_VAULT.md`
- [ ] Crear `DEPLOYMENT_GUIDE.md`
- [ ] Escribir `RAYDIUM_DEEP_DIVE.md` después del Sprint 2

### Código
- [x] Implementar Executor Trait
- [x] Sistema de observabilidad con tracing
- [ ] Refactorizar JupiterExecutor para usar el trait
- [ ] Completar RaydiumExecutor (Sprints 1-4)
- [ ] Integración gRPC funcional
- [ ] Migración a SQLite

---

## 📞 Contacto

Para preguntas sobre The Blue Book:
- **Autor:** Ruben
- **Proyecto:** higrub89/SOL_BOT
- **Mentoría:** Ingeniería de Sistemas Críticos (2026-02-09)

---

**Versión del Blue Book:** 1.0.0  
**Última Actualización:** 2026-02-09 22:21 UTC  
**Commits Totales en el Proyecto:** 2+ (en crecimiento)

---

> "El que controla la documentación, controla el conocimiento. El que controla el conocimiento, construye sistemas inmortales." 🏎️
