# ╔═══════════════════════════════════════════════════════════════════════╗
# ║                   THE CHASSIS - PRODUCTION IMAGE                      ║
# ║          Multi-Stage Build with Cargo-Chef (~30s rebuild)             ║
# ╚═══════════════════════════════════════════════════════════════════════╝

FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-json recipe.json

FROM chef AS builder 
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    libprotobuf-dev \
    libssl-dev \
    pkg-config \
    make \
    && rm -rf /var/lib/apt/lists/*

COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the layer that takes time and will be cached
RUN cargo chef cook --release --recipe-json recipe.json

# Build application
COPY . .
RUN cargo build --release --workspace

# --- Etapa 2: Runtime ---
FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/the_chassis_app /app/the_chassis_app

ENV RUST_LOG=info
ENV TZ=UTC

CMD ["./the_chassis_app", "monitor"]
