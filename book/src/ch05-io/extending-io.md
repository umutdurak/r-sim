# Extending the I/O Layer

The I/O subsystem is designed to be extensible. Adding support for a new protocol or hardware interface follows a consistent pattern.

## Steps to Add a New I/O Type

1. **Define the config struct:**
   ```rust
   #[derive(Debug, Deserialize, Clone)]
   pub struct CanConfig {
       pub name: String,
       pub interface: String,
       pub bitrate: u32,
   }
   ```

2. **Add a `TaskConfig` variant:**
   ```rust
   pub enum TaskConfig {
       // ... existing variants
       Can(CanConfig),
   }
   ```

3. **Implement the task struct** with `SimulationTask` trait, including the I/O methods (`initialize_io`, `read_inputs`, `write_outputs`)

4. **Register in the factory** so TOML configs with `type = "Can"` are recognized

## Design Principles

- **Simulated by default** — All I/O tasks work without real hardware for testing
- **Interface stability** — The `SimulationTask` trait never changes; new I/O types just implement it
- **Declarative config** — Users define I/O through TOML, not code

This extensibility makes r-sim suitable for domains beyond its built-in protocols — CAN bus for automotive, SpaceWire for aerospace, OPC UA for Industry 4.0, and any protocol you need.
