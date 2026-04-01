# ╔═══════════════════════════════════════════════════════════════════════╗
# ║                   THE CHASSIS - GCP OPTIMIZED IMAGE                   ║
# ║          Multi-Stage Build with Cargo-Chef & Google CLI               ║
# ╚═══════════════════════════════════════════════════════════════════════╝

FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    libprotobuf-dev \
    libssl-dev \
    pkg-config \
    make \
    cmake \
    g++ \
    && rm -rf /var/lib/apt/lists/*

COPY --from=planner /app/recipe.json recipe.json
# Build dependencies
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release --workspace

# --- Etapa 2: Runtime ---
FROM ubuntu:24.04
WORKDIR /app

# Instalar dependencias necesarias y el gcloud CLI para Secret Manager
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    procps \
    curl \
    gnupg \
    && curl https://packages.cloud.google.com/apt/doc/apt-key.gpg | gpg --dearmor -o /usr/share/keyrings/cloud.google.gpg \
    && echo "deb [signed-by=/usr/share/keyrings/cloud.google.gpg] https://packages.cloud.google.com/apt cloud-sdk main" | tee -a /etc/apt/sources.list.d/google-cloud-sdk.list \
    && apt-get update && apt-get install -y google-cloud-cli \
    && rm -rf /var/lib/apt/lists/*

# Copiar el binario compilado y configuración
COPY --from=builder /app/target/release/the_chassis_app /app/the_chassis_app
COPY operational/data/settings.json /app/settings.json

# Configuración de entorno
ENV RUST_LOG=info
ENV TZ=UTC
ENV HUNTER_MODE=devnet

# El bot necesita permisos para SCHED_FIFO, se maneja en el run command de docker/gce
CMD ["./the_chassis_app", "monitor"]
