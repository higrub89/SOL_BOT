---
description: Auditoría de seguridad pre-deploy para The Chassis
mode: agent
---

Auditoría completa antes de deploy a GCP:

1. Secrets: escanear todos los archivos por keys o tokens hardcodeados
2. Dependencias: cargo audit y verificar versiones críticas
3. Docker: verificar que la imagen no expone puertos innecesarios
4. Env vars: confirmar que vars críticas están en GCP Secret Manager
5. Permisos: archivos con secrets tienen chmod 600
6. Git: .gitignore cubre todos los archivos sensibles
7. Memoria: ejecutar hft-memory-profiler y reportar cualquier leak

Reporte: tabla con Ítem / Estado (✅/⚠️/❌) / Acción requerida.
