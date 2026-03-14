---
applyTo: ["**/*.ts", "**/*.tsx", "**/*.mts"]
---

- strict: true en tsconfig sin excepciones
- Validación en boundaries: zod siempre en entrada de datos externos
- No enums nativos → const assertions o union types discriminados
- Async: prefer async/await, manejo exhaustivo de errores
- No any implícito ni explícito — usar unknown + type guard
- tsconfig: module NodeNext o ESNext, imports con extensión explícita en ESM
