---
name: ml-signal
description: >
  Use this skill when working in intelligence/: ML model training, signal
  validation, dataset management, or research notebooks.
metadata:
  scope:
    - intelligence/
  auto_invoke:
    - "Train a model"
    - "Validate a signal"
    - "Add to dataset"
    - "Run ML script"
    - "Connect Python to Rust engine"
---

# Skill: ML Signal Research

## Intelligence Directory Structure

```
intelligence/
├── datasets/
│   ├── raw/          → Unprocessed data (never modify)
│   └── processed/    → Feature-engineered, ready for training
├── ml/
│   ├── models/       → Saved model weights (.pt, .onnx)
│   └── scripts/
│       └── operational/audits/  → Model performance reports
├── scripts/          → Data collection, feature engineering
└── src/              → Library code shared across scripts
```

## Model Validation Gate

Before merging any model change, create a validation report in
`intelligence/ml/scripts/operational/audits/`:

```markdown
# Model Validation: {model_name} — {date}

**Sharpe Ratio:** ≥ 1.5 required
**Max Drawdown:** ≤ 15% required
**Win Rate:** ≥ 52% required
**Backtest Period:** minimum 90 days
**Dataset:** describe source and date range
```

---

## Signal → Engine IPC Strategy

The Python intelligence layer communicates with the Rust engine via
serialized Protobuf messages (`core/proto/signal.proto`).

**Choose the IPC transport based on observed latency budget:**

### Tier 1 — ZMQ TCP (Default · ~50–200µs localhost)
Use for initial development and when processes may run on different hosts.

```python
import zmq, time
from core.generated import signal_pb2

ctx = zmq.Context()
socket = ctx.socket(zmq.PUSH)
socket.connect("tcp://127.0.0.1:5555")   # env: ZMQ_SIGNAL_SOCKET

signal = signal_pb2.Signal(
    pair="SOL/USDC",
    direction=signal_pb2.Direction.LONG,
    confidence=0.87,
    timestamp_ms=int(time.time() * 1000),
)
socket.send(signal.SerializeToString())
```

### Tier 2 — Unix Domain Sockets (UDS · ~5–20µs)
Drop-in replacement when Python and Rust run on the same machine.
Eliminates TCP stack overhead with no code changes beyond the address:

```python
socket.connect("ipc:///tmp/chassis_signal.sock")   # ZMQ IPC transport
```

```rust
// Rust side — same change
let socket_addr = std::env::var("CHASSIS_IPC_SOCKET")
    .unwrap_or_else(|_| "ipc:///tmp/chassis_signal.sock".into());
```

### Tier 3 — Shared Memory / `mmap` (· ~500ns–2µs)
Use only when Tier 2 latency is still a bottleneck (measured, not assumed).
Requires a lock-free ring buffer. Reference implementation:
`core/src/sensors/shm_receiver.rs` (create when needed).

```
Python writer → mmap ring buffer → Rust reader
              ↑ lock-free, cache-line aligned
```

**Decision rule:** measure first with `cargo bench` + `hyperfine`.
Never migrate to a higher tier without a benchmark proving the gain.

---

## IPC Tier Environment Variables

```bash
# .env
ZMQ_SIGNAL_SOCKET=tcp://127.0.0.1:5555   # Tier 1
# ZMQ_SIGNAL_SOCKET=ipc:///tmp/chassis_signal.sock  # Tier 2
CHASSIS_IPC_MODE=zmq   # zmq | uds | shm
```
