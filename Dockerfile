# ╔═══════════════════════════════════════════════════════════════════════╗
# ║                   THE CHASSIS - PRODUCTION IMAGE                      ║
# ║          Multi-Stage Build with Dependency Caching (~1min rebuild)    ║
# ╚═══════════════════════════════════════════════════════════════════════╝

# --- Etapa 1: Builder (Compilación) ---
FROM rust:latest as builder

# Instalar dependencias del sistema necesarias para compilar
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    libprotobuf-dev \
    libssl-dev \
    pkg-config \
    make \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# ═══ TRUCO DE CACHÉ: Primero solo Cargo.toml + dummy src ═══
# Así Docker cachea la compilación de las ~370 dependencias
# y solo recompila TU código cuando cambias archivos .rs

# 1. Copiar solo los manifiestos
COPY Cargo.toml Cargo.lock ./
COPY core/the_chassis/Cargo.toml core/the_chassis/Cargo.toml
COPY intelligence/Cargo.toml intelligence/Cargo.toml

# 2. Crear archivos dummy para que cargo compile las dependencias
RUN mkdir -p core/the_chassis/src/bin && \
    echo "fn main() {}" > core/the_chassis/src/bin/main.rs && \
    echo "// dummy" > core/the_chassis/src/lib.rs && \
    mkdir -p intelligence/src && \
    echo "// dummy" > intelligence/src/lib.rs

# 3. Compilar SOLO dependencias (esto se cachea en Docker layer)
RUN cargo build --release --workspace 2>/dev/null || true

# 4. Eliminar los artefactos dummy (para que cargo recompile nuestro código)
RUN rm -rf core/the_chassis/src intelligence/src target/release/.fingerprint/the_chassis* target/release/.fingerprint/intelligence*

# ═══ AHORA SÍ: Copiar código real y compilar (~30-60 seg) ═══
COPY core ./core
COPY intelligence ./intelligence
COPY Makefile ./

RUN cargo build --release --workspace

# --- Etapa 2: Runtime (Ejecución Ligera) ---
FROM debian:bookworm-slim

WORKDIR /app

# Instalar librerías runtime (SSL, certificados CA)
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copiar el binario compilado desde la etapa builder
COPY --from=builder /app/target/release/the_chassis_app /app/the_chassis_app

# Configurar entorno
ENV RUST_LOG=info
ENV TZ=UTC

# Comando por defecto: Monitor Mode
CMD ["./the_chassis_app", "monitor"]
