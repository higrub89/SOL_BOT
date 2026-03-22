---
name: skill-sync
description: Reglas y responsabilidades para regenerar las directivas auto-invoke de los Agentes de IA en AGENTS.md.
auto-invokes:
  - Después de crear o modificar un archivo SKILL.md
  - Regenerar la tabla auto-invoke de AGENTS.md
---
# Skill-Sync SOP

1. Este skill debe aplicarse cuando se invoca el archivo `skills/skill-sync.sh` o `scripts/sync.sh` (o cuando un AI agent necesita actualizar autoinvokes).
2. El script de sincronización leerá la sección frontmatter `auto-invokes` de todos los archivos `skills/*/SKILL.md` y construirá la tabla `| Action | Skill |`.
3. Inyectará el Markdown directamente en el `AGENTS.md` de la raíz, actualizándolo de esta manera para mantener una visión monolítica de auto-triggers aeronáuticos.
