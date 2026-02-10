# ╔═══════════════════════════════════════════════════════════════════════╗
# ║                   THE CHASSIS - PRODUCTION IMAGE                      ║
# ║             Multi-Stage Build for Optimized Runtime                   ║
# ╚═══════════════════════════════════════════════════════════════════════╝

# --- Etapa 1: Builder (Compilación) ---
# Usamos la imagen oficial de Rust moderna
FROM rust:latest as builder

# Instalar dependencias del sistema necesarias para compilar (Protobuf, SSL)
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    libprotobuf-dev \
    libssl-dev \
    pkg-config \
    make \
    && rm -rf /var/lib/apt/lists/*

# Crear estructura de workspace
WORKDIR /app
COPY Cargo.toml Cargo.lock Makefile ./
COPY core ./core
COPY intelligence ./intelligence

# Compilar en modo release (optimizaciones máximas)
RUN cargo build --release --workspace

# --- Etapa 2: Runtime (Ejecución Ligera) ---
# Usamos bookworm-slim (OpenSSL 3) para coincidir con la imagen de compilación
FROM debian:bookworm-slim

WORKDIR /app

# Instalar librerías runtime (SSL, certificados CA)
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copiar el binario compilado desde la etapa builder
COPY --from=builder /app/target/release/the_chassis_app /app/the_chassis_app

# NOTA: targets.json y pools_cache.json se montan como volúmenes en docker-compose.yml
# NOTA: .env NO se copia por seguridad. Se debe inyectar en deployment.

# Configurar entorno
ENV RUST_LOG=info
ENV TZ=UTC

# Comando por defecto: Monitor Mode
CMD ["./the_chassis_app", "monitor"]
