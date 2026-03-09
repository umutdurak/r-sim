# Reference

## CLI Reference

```
r-sim [COMMAND]

Commands:
  run       Run the simulation
  control   Control the simulation status
  scenario  Manage simulation scenarios
  help      Print this message

Run Options:
  -c, --config-file <FILE>           Configuration file path
  -s, --simulation-duration-secs <N> Duration in seconds
  -t, --time-step-millis <N>         Time step in milliseconds

Control Options:
  start, pause, resume, stop

Scenario Options:
  save <NAME> -c <CONFIG>
  load <NAME> -s <SECS> -t <MS>
  list
```

## Web API Reference

| Endpoint | Method | Request | Response |
|----------|--------|---------|----------|
| `/data` | GET | — | `{ current_time_secs, task_execution_times_micros }` |
| `/graph` | GET | — | `{ tasks: [], dependencies: [] }` |
| `/parameters` | GET | — | `{ task_name: { param_name: value } }` |
| `/parameters/set` | POST | `{ task_name, param_name, param_value }` | `"Parameter set successfully."` |
| `/control?cmd=X` | GET | `cmd=pause\|resume\|stop` | Status message |

## Configuration Reference

### Global Parameters
| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `time_multiplier` | float | 1.0 | Simulation speed scale factor |

### Task Types
| Type | Required Fields | Optional Fields |
|------|----------------|-----------------|
| `Custom` | `name` | — |
| `Fmu` | `name`, `path` | — |
| `Gpio` | `name`, `pins` | — |
| `Serial` | `name`, `port`, `baud_rate` | — |
| `Udp` | `name`, `local_addr`, `remote_addr` | — |
| `Analog` | `name`, `channels`, `is_input` | `sampling_rate_hz` |
| `ModbusTcp` | `name`, `ip_address`, `port` | — |

### Dependency Fields
| Field | Required | Values |
|-------|----------|--------|
| `from` | Yes | Task name |
| `to` | Yes | Task name |
| `type` | Yes | `"direct"`, `"memory_block"` |
| `data_flow` | Yes | `"output_to_input"` |

### Logging Fields
| Field | Type | Description |
|-------|------|-------------|
| `log_file` | string | Output CSV path |
| `log_interval_millis` | integer | Logging interval in ms |
| `logged_outputs` | table | `{ TaskName = ["signal1", "signal2"] }` |

## Glossary

| Term | Definition |
|------|-----------|
| **Co-simulation** | Combining multiple simulation models into one coordinated execution |
| **DAG** | Directed Acyclic Graph — a graph with no cycles |
| **FMI** | Functional Mock-up Interface — standardized model exchange format |
| **FMU** | Functional Mock-up Unit — a packaged simulation model |
| **HIL** | Hardware-in-the-Loop — testing hardware controllers against simulated plants |
| **MIL** | Model-in-the-Loop — all-simulation testing |
| **Overrun** | When a time step's computation exceeds the time step duration |
| **SIL** | Software-in-the-Loop — testing production software against simulated plants |
| **Time step** | The fixed interval between simulation loop iterations |
| **Topological sort** | Algorithm to find a linear ordering of a DAG |
| **XiL** | X-in-the-Loop — generalized testing methodology |
