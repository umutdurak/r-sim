# r-sim: Real-Time Co-Simulation Framework

A modular, real-time co-simulation framework written in Rust for hardware-in-the-loop (HIL) and software-in-the-loop (SIL) testing. r-sim provides deterministic task scheduling, FMU integration, multi-protocol I/O support, and a web-based monitoring interface.

> **Developed with [Antigravity](https://deepmind.google/) and [Opus](https://www.anthropic.com/)** — AI-assisted pair programming tools by Google DeepMind and Anthropic.

## Features

- **Task-based simulation engine** — Periodic execution with configurable time steps and topological dependency ordering
- **FMU co-simulation** — Load and execute external models via dynamic libraries (`cdylib`)
- **Multi-protocol I/O** — GPIO, Serial, UDP, Analog, Modbus TCP with synchronized read/write cycles
- **Causal loop detection** — Automatic cycle detection with memory block resolution
- **Web monitoring** — Real-time `/data`, `/graph`, and `/parameters` endpoints via Warp
- **Lifecycle control** — HTTP-based pause/resume/stop control
- **TOML configuration** — Declarative simulation setup with task definitions, dependencies, and logging
- **Scenario management** — Save, load, and list simulation configurations
- **CSV data logging** — Configurable output logging with selectable signals
- **Custom task API** — Extensible `SimulationTask` trait for user-defined components via `TaskFactory`

## Quick Start

```bash
# Build the framework and test FMU
cargo build

# Run with default config
./target/debug/r-sim run -c default_config.toml -s 10 -t 100

# Run with custom duration and time step
./target/debug/r-sim run -c my_config.toml --simulation-duration-secs 60 --time-step-millis 50
```

## CLI Commands

```bash
r-sim run -c <CONFIG> -s <SECS> -t <MS>    # Run simulation
r-sim control start|pause|resume|stop       # Lifecycle control
r-sim scenario save <NAME> -c <CONFIG>      # Save scenario
r-sim scenario load <NAME> -s <SECS> -t <MS> # Load and run scenario
r-sim scenario list                         # List saved scenarios
```

## Configuration Example

```toml
time_multiplier = 1.0

[[tasks]]
type = "Fmu"
name = "PlantModel"
path = "./target/debug/deps/libfmu_test.dylib"

[[tasks]]
type = "Custom"
name = "Controller"

[[tasks]]
type = "Gpio"
name = "SensorInput"
pins = [1, 2, 3]

[[dependencies]]
from = "SensorInput"
to = "Controller"
type = "direct"
data_flow = "output_to_input"

[[dependencies]]
from = "Controller"
to = "PlantModel"
type = "direct"
data_flow = "output_to_input"

[logging]
log_file = "simulation_log.csv"
log_interval_millis = 100
logged_outputs = { Controller = ["custom_output"] }
```

## Web API

While running, the framework exposes endpoints on `http://127.0.0.1:3030`:

| Endpoint | Method | Description |
|---|---|---|
| `/data` | GET | Simulation time, task execution times |
| `/graph` | GET | Task graph with dependencies |
| `/parameters` | GET | All task parameters |
| `/parameters/set` | POST | Update a task parameter at runtime |
| `/control?cmd=pause\|resume\|stop` | GET | Lifecycle control |

## Testing

```bash
# Run all 26 integration tests
cargo test --test integration_test -- --test-threads=1
```

See [test_plan.sdoc](test_plan.sdoc) for the full test plan covering 29 test cases across core simulation, FMU, I/O, configuration, robustness, and advanced features.

## Project Structure

```
r-sim/
├── src/main.rs              # CLI entry point
├── framework/src/lib.rs     # Core simulation engine
├── fmu_test/src/lib.rs      # Test FMU (cdylib)
├── tests/
│   ├── integration_test.rs  # 26 Rust integration tests
│   └── run_tests.sh         # Shell-based test runner
├── default_config.toml      # Example configuration
└── test_plan.sdoc           # Requirements-traced test plan
```

## License

See [LICENSE](LICENSE) for details.
