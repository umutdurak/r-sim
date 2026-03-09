# Installation

## Prerequisites

r-sim is written in Rust, so you'll need the Rust toolchain installed. If you don't have it yet:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Verify your installation:

```bash
rustc --version
cargo --version
```

You should see Rust 1.70 or later. r-sim also requires:

- **Git** for cloning the repository
- A **text editor** (VS Code with rust-analyzer is recommended)
- **curl** or a web browser for interacting with the monitoring interface

## Cloning and Building r-sim

Clone the r-sim repository and build the framework:

```bash
git clone https://github.com/umutdurak/r-sim.git
cd r-sim
cargo build
```

This compiles the following workspace members:

| Crate | Purpose |
|-------|---------|
| `r-sim` | Command-line interface and simulation runner |
| `framework` | Core simulation engine library |
| `fmu_test` | Example FMU (dynamic library) for testing |

After building, verify the binary:

```bash
./target/debug/r-sim --help
```

You should see:

```
Usage: r-sim [COMMAND]

Commands:
  run       Run the simulation
  control   Control the simulation status
  scenario  Manage simulation scenarios
  help      Print this message or help for subcommands
```

## Project Structure

```
r-sim/
├── src/main.rs              # CLI entry point
├── framework/
│   └── src/lib.rs           # Core simulation engine
├── fmu_test/
│   └── src/lib.rs           # Example external model (cdylib)
├── tests/
│   ├── integration_test.rs  # Rust integration tests
│   └── test_configs/        # Test configuration files
├── default_config.toml      # Example TOML configuration
├── book/                    # This book
├── Cargo.toml               # Workspace manifest
└── README.md
```

> **Note:** The `framework` crate contains all of r-sim's core functionality — task definitions, the simulation graph, the web server, and configuration parsing. The top-level `r-sim` crate is a thin CLI wrapper.
