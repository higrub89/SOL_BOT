---
name: trading-pr-gate
description: Convenciones de revisión y seguridad para Pull Requests.
auto-invokes:
  - Redactar templates de Pull Requests
  - Aprobar integraciones hacia master/main
---
# Pull Request Gate (Aerospace Security)

1. El PR requiere adjuntar un reporte del impacto en latencia o la inexistencia de memory-leaks.
2. Todo Pull Request hacia `master` debe ser revisado manualmente.
3. El Checklist del PR DEBE incluir la verificación de variables globales mutables (que no debe haberlas) y que ningún endpoint de RPC tenga endpoints `public` inyectados en producción en lugar de los RPC/Geyser seguros de Helius.
