---
applyTo: ["**/*.c", "**/*.cpp", "**/*.h", "**/*.hpp"]
---

- Norminette activa en proyectos bajo ~/Workspace/42_universe/
- Flags obligatorios: -Wall -Wextra -Werror. Cero warnings
- Sin variables globales excepto donde la Norma lo permita
- Gestión de memoria explícita: cada malloc tiene su free documentado
- Valgrind clean: zero leaks, zero errors en entrega
- Headers: include guards (#ifndef), sin includes circulares
- Fuera de 42: -O3 -march=native para código de rendimiento
