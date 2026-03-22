# AGENTS.md — MISSION CONTROL

## 1. SYSTEM DIRECTIVE (Obligatoria – Léase primero)

**ADVERTENCIA CRÍTICA PARA CUALQUIER AGENTE (Claude, Cursor, Gemini, Codex, etc.):**
Este repositorio **NO** es SaaS, MVP ni prototipo educativo.  
Es infraestructura financiera de misión crítica (nivel HRT / Jane Street / Palantir Delta).  
Tolerancia cero a fallos. Cualquier violación de las reglas siguientes = fallo de misión.

## 2. DOCTRINE – Principios No Negociables

1. **Minimalismo implacable**  
   - Menos dependencias > dependencias "mejores".  
   - Menos líneas > más líneas elegantes.  
   - Superficie de ataque mínima en hot-path.

2. **Ejecución determinista + Hardware sympathy**  
   - Hot-path (Rust): zero-allocation estricto, pre-asignación, no `unwrap()`, no `panic`.  
   - Errores: siempre `Result<_, CriticalError>` + graceful recovery.  
   - Latencia objetivo: < 100 µs end-to-end (Geyser → sign → Jito).

3. **Estética suiza + precisión geométrica**  
   - Código, logs, commits, CI/CD: simetría clínica.  
   - Nombres: precisión quirúrgica (sin abreviaturas ambiguas).  
   - Formato 100% determinista: `cargo fmt`, `clippy`.

4. **Silencio táctico**  
   - Respuestas: densas, técnicas, sin disculpas ni verbosidad.  
   - Solo valor militar: analiza → propone → ejecuta.

## 3. REGLAS OPERATIVAS Y LÍNEAS ROJAS

- **Nunca toques ni leas**: `operational/wallets/`, `.env`, keys, seeds. (Carpeta air-gapped).
- **Hot-path**: Rust (Tokio + tonic + zerocopy). Python solo se usa para research offline.
- **Comunicación cerebro ↔ chasis**: Unix Domain Sockets o Shared memory (NO usar ZMQ en localhost crítico).
- **Rechazo Automático**: Proponer código con `unwrap()`, inyectar código no determinista en Rust, o leer el disco dentro de un hilo de alta frecuencia.

## 4. ESTRUCTURA CANÓNICA

| Directory       | Purpose                                         | Language |
|-----------------|-------------------------------------------------|----------|
| `core/`         | Low-latency execution engine (The Chassis)      | Rust     |
| `intelligence/` | ML models, datasets, signal research            | Python   |
| `operational/`  | Wallets, audits, DevOps scripts                 | Shell    |
| `skills/`       | AI agent skills (Micro-skills HFT)              | Markdown |
| `mcp/`          | MCP schemas (Migrating to Node Zod Server)      | Markdown |

## 5. CAPACIDADES DINÁMICAS (Agent Skills)

### Auto-invoke Skills
Cuando ejecutes estas acciones, **SIEMPRE** carga la Skill listada antes de actuar:

<!-- AUTO-INVOKE-START -->
| Action | Skill |
|--------|-------|
| - Aprobar integraciones hacia master/main | `trading-pr-gate` |
| Calcular swap_amount_out o predecir precios en Raydium | `raydium-v2` |
| - Call Solana RPC, send transactions, parse slots | `solana-jito-mev` |
| - Cambios en módulos de ejecución de Jupiter/Raydium | `solana-jito-mev` |
| - Configurar GitHub Actions o flujos automatizados (CI) | `trading-ci` |
| - Crear un git commit | `trading-commit` |
| - Después de crear o modificar un archivo SKILL.md | `skill-sync` |
| - Edit any .rs file in core/ | `rust-hft-patterns` |
| - Evaluar impacto de performance | `trading-ci` |
| Leer, parsear o interactuar con pools de Raydium V2/V4 | `raydium-v2` |
| - Modificar o añadir features o fixes al log de git | `trading-commit` |
| - Modifying the ExecutionRouter | `rust-hft-patterns` |
| - Redactar templates de Pull Requests | `trading-pr-gate` |
| - Regenerar la tabla auto-invoke de AGENTS.md | `skill-sync` |
<!-- AUTO-INVOKE-END -->

## 6. SÍNTESIS DE SUBSISTEMAS

- [`core/AGENTS.md`](core/AGENTS.md) — Rust engine patterns (Zero-Alloc, Lock-Free).
- [`intelligence/AGENTS.md`](intelligence/AGENTS.md) — ML workflows y puentes deterministas.
- [`operational/AGENTS.md`](operational/AGENTS.md) — Wallet security, MCP Zod gateways.
