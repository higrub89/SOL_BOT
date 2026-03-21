# ╔═══════════════════════════════════════════════════════════════════════╗
# ║                  THE CHASSIS - MASTER MAKEFILE                        ║
# ║             Institutional Solana Trading Engine v2.0                  ║
# ╚═══════════════════════════════════════════════════════════════════════╝

# Configuración del entorno
CARGO := cargo
TARGET := target/release/the_chassis_app

.PHONY: all build check test clean run monitor scan backtest help

# --------------------------
# COMANDOS PRINCIPALES
# --------------------------

all: check build test ## Ejecuta todo el pipeline de CI local (check -> build -> test)

check: ## Verifica errores de compilación sin generar binarios
	@echo "🔍 Verificando código..."
	@$(CARGO) check --workspace

build: ## Compila el proyecto en modo release (optimizado)
	@echo "🏗️  Compilando The Chassis (Release Mode)..."
	@$(CARGO) build --release --workspace

test: ## Ejecuta todos los tests unitarios y de integración
	@echo "🧪 Ejecutando tests..."
	@$(CARGO) test --workspace

clean: ## Limpia los artefactos de compilación
	@echo "🧹 Limpiando target/..."
	@$(CARGO) clean

# --------------------------
# EJECUCIÓN DEL BOT
# --------------------------

run: build ## Ejecuta el bot en modo monitor por defecto
	@echo "🚀 Iniciando The Chassis..."
	@./$(TARGET) monitor

monitor: build ## Lanza el modo Monitor (Trading Automatizado)
	@echo "👁️  Iniciando Monitor de Trading..."
	@./$(TARGET) monitor

scan: build ## Lanza el Scanner de Red (Pump.fun Sensor)
	@echo "📡 Iniciando Scanner de Red..."
	@./$(TARGET) scan

buy: build ## Compra rápida (Uso: make buy MINT=... SOL=...)
	@echo "💸 Ejecutando compra rápida..."
	@./$(TARGET) buy --mint $(MINT) --sol $(SOL)

# --------------------------
# DESARROLLO & INTELLIGENCE
# --------------------------

backtest: ## Ejecuta el suite de backtesting de estrategias
	@echo "📉 Corriendo Backtests..."
	@$(CARGO) test --package intelligence_rs --lib tests::backtest_simulation -- --nocapture

lint: ## Ejecuta el linter (Clippy) para asegurar calidad de código
	@echo "💅 Pasando linter..."
	@$(CARGO) clippy --workspace -- -D warnings

format: ## Formatea el código automáticamente
	@echo "auto-format..."
	@$(CARGO) fmt

# --------------------------
# UTILIDADES
# --------------------------

help: ## Muestra esta ayuda
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

# Alias para comodidad
b: build
r: run
t: test
