---
description: Refactor y hardening de módulo crítico existente
mode: agent
---

Actúa como revisor de seguridad y rendimiento para este módulo:

1. Detecta:
   - unwrap()/expect() peligrosos y panics potenciales
   - Allocaciones innecesarias en hot paths
   - Uso incorrecto de lifetimes o concurrencia

2. Propón cambios concretos:
   - Reemplazos de tipos y patrones de error handling
   - Cambios en estructuras para reducir allocations

3. Genera diff propuesto en formato patch unificado

Prioriza seguridad y latencia. Mantén la API pública estable.
