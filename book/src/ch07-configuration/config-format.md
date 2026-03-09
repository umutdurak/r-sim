# Configuration File Format

A TOML configuration file defines the complete simulation setup: tasks, dependencies, logging, and global parameters.

## Complete Example

```toml
# simulation_config.toml

# Global parameters
time_multiplier = 1.0

# Task definitions
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

# Dependencies
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

# Logging
[logging]
log_file = "simulation_log.csv"
log_interval_millis = 100
logged_outputs = { Controller = ["custom_output"] }
```

## Structure Overview

| Section | TOML Syntax | Purpose |
|---------|-------------|---------|
| Global params | Top-level keys | `time_multiplier`, etc. |
| Tasks | `[[tasks]]` | Array of task definitions |
| Dependencies | `[[dependencies]]` | Array of task connections |
| Logging | `[logging]` | CSV output configuration |

> **Note:** In TOML, `[[array_name]]` denotes an array of tables. Each `[[tasks]]` block adds one task to the array. This is different from `[table_name]`, which defines a single table.
