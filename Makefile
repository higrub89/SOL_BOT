# ╔═══════════════════════════════════════════════════════════════════════╗
# ║              THE CHASSIS — MASTER MAKEFILE v2.1                       ║
# ║         Institutional Solana Trading Engine                           ║
# ╚═══════════════════════════════════════════════════════════════════════╝

.PHONY: all build release check test audit lint format clean clean-deep \
        proto agents \
        run monitor scan buy \
        status logs-live check-apis tune-gcp backtest \
        docker-up docker-down docker-logs \
        help b r t

# ── Paths ────────────────────────────────────────────────────
CARGO   := cargo
TARGET  := target/release/the_chassis_app
SCRIPTS := ./operational/scripts
DEPLOY  := ./operational/deploy
DATA    := ./operational/data

# ── Default ──────────────────────────────────────────────────
all: check build test

# ── Build ────────────────────────────────────────────────────
build: ## Debug build (workspace)
	@echo "🏗️  Building (debug)..."
	@$(CARGO) build --workspace

release: ## Optimized release build
	@echo "🚀 Building (release)..."
	@$(CARGO) build --release --workspace

check: ## Type-check without producing binaries
	@echo "🔍 Checking..."
	@$(CARGO) check --workspace

# ── Quality Gates ────────────────────────────────────────────
lint: ## Clippy + fmt check
	@$(CARGO) clippy --workspace -- -D warnings
	@$(CARGO) fmt --check

format: ## Auto-format code
	@$(CARGO) fmt

test: ## Run test suite
	@$(CARGO) test --workspace

audit: ## Security audit (cargo-audit)
	@$(CARGO) audit

backtest: ## Run backtesting suite
	@$(CARGO) test --package intelligence_rs --lib tests::backtest_simulation -- --nocapture

# ── Protobuf ─────────────────────────────────────────────────
proto: ## Regenerate Rust + Python Protobuf bindings
	@echo "🔧 Regenerating Protobuf bindings..."
	@$(CARGO) build --workspace
	@echo "   Rust bindings → core/src/generated/"
	@if command -v python3 >/dev/null 2>&1; then \
	  python3 -m grpc_tools.protoc \
	    -I core/proto \
	    --python_out=intelligence/src \
	    --pyi_out=intelligence/src \
	    core/proto/signal.proto && \
	  echo "   Python bindings → intelligence/src/"; \
	else \
	  echo "   [SKIP] grpc_tools not found — pip install grpcio-tools"; \
	fi

# ── Bot Execution ────────────────────────────────────────────
run: release ## Build + run in monitor mode
	@echo "🚀 Starting The Chassis..."
	@./$(TARGET) monitor

monitor: release ## Trading monitor mode (automated)
	@echo "👁️  Starting Monitor..."
	@./$(TARGET) monitor

scan: release ## Network scanner (pump.fun sensor)
	@echo "📡 Starting Scanner..."
	@./$(TARGET) scan

buy: release ## Quick buy (Usage: make buy MINT=... SOL=...)
	@echo "💸 Executing buy..."
	@./$(TARGET) buy --mint $(MINT) --sol $(SOL)

# ── Operational ──────────────────────────────────────────────
status: ## Engine status + recent logs
	@$(SCRIPTS)/bot_manager.sh status

logs-live: ## Tail logs in real time
	@$(SCRIPTS)/bot_manager.sh logs

check-apis: ## Health check (Helius, Jupiter, Telegram, Jito)
	@python3 $(SCRIPTS)/check_apis.py

tune-gcp: ## Kernel tuning for GCP (requires sudo)
	@chmod +x $(SCRIPTS)/setup_gcp.sh && sudo $(SCRIPTS)/setup_gcp.sh

# ── Docker ───────────────────────────────────────────────────
docker-up: ## Launch bot via docker-compose (production)
	docker compose -f $(DEPLOY)/docker-compose.yml up -d
	docker compose -f $(DEPLOY)/docker-compose.yml logs -f

docker-down: ## Stop docker-compose services
	docker compose -f $(DEPLOY)/docker-compose.yml down

docker-logs: ## Follow docker logs
	docker compose -f $(DEPLOY)/docker-compose.yml logs -f

# ── Agent Workspace ──────────────────────────────────────────
agents: ## Setup AI agent skills (Claude Code / Cursor / Codex)
	@echo "🤖 Setting up AI agent skills..."
	@./skills/setup.sh

# ── Cleanup ──────────────────────────────────────────────────
clean: ## Remove Rust build artifacts
	@$(CARGO) clean

clean-deep: clean ## Deep clean: Rust + Python caches + logs
	@echo "🧹 Deep clean..."
	@find . -type d -name "__pycache__" -not -path "./.git/*" -exec rm -rf {} + 2>/dev/null || true
	@find . \( -name "*.pyc" -o -name "*.pyo" \) -not -path "./.git/*" -delete 2>/dev/null || true
	@find . -name ".pytest_cache" -not -path "./.git/*" -exec rm -rf {} + 2>/dev/null || true
	@find . -name "*.egg-info" -not -path "./.git/*" -exec rm -rf {} + 2>/dev/null || true
	@find ./operational/logs -name "*.log" -o -name "*.jsonl" -delete 2>/dev/null || true
	@find ./logs -name "*.log" -delete 2>/dev/null || true
	@find ./target -name "incremental" -type d -exec rm -rf {} + 2>/dev/null || true
	@echo "   ✓ Done. Run 'make build' for a clean compile."

# ── Aliases ──────────────────────────────────────────────────
b: build
r: run
t: test

# ── Help ─────────────────────────────────────────────────────
help: ## Show this help
	@echo ""
	@echo "  The Chassis — Command Center"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
	  awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-18s\033[0m %s\n", $$1, $$2}'
	@echo ""
