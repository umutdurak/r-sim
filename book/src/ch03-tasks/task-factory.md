# The TaskFactory Pattern

r-sim uses the **Factory Pattern** to create tasks from configuration. The `TaskFactory` maintains a registry of task type names mapped to constructor functions, enabling the framework to instantiate any registered task type from a TOML configuration file.

## How It Works

```rust
pub struct TaskFactory {
    creators: HashMap<&'static str, TaskCreator>,
}

pub type TaskCreator = fn(TaskConfig) -> Result<Box<dyn SimulationTask>, FrameworkError>;
```

At startup, the factory is populated with all built-in task types:

```rust
let mut factory = TaskFactory::new();
// Built-in types are registered automatically
```

When parsing the configuration, the factory matches the `type` field:

```toml
[[tasks]]
type = "Custom"    # ← looked up in the factory registry
name = "MyTask"
```

```rust
// The factory creates the right task based on the "type" string
let task = factory.create_task(task_config)?;
```

## The TaskConfig Enum

Configuration is deserialized into a `TaskConfig` enum that captures all possible task types:

```rust
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum TaskConfig {
    Fmu(FmuConfig),
    Gpio(GpioConfig),
    Serial(SerialConfig),
    Udp(UdpConfig),
    Analog(AnalogConfig),
    ModbusTcp(ModbusTcpConfig),
    Custom(CustomConfig),
}
```

The `#[serde(tag = "type")]` attribute tells Serde to use the `type` field as a discriminator. When you write:

```toml
[[tasks]]
type = "Gpio"
name = "Sensors"
pins = [1, 2, 3]
```

Serde automatically deserializes this into `TaskConfig::Gpio(GpioConfig { name: "Sensors", pins: [1, 2, 3] })`.

## Extending the Factory

To add a new task type, you would:

1. Define a new config struct (e.g., `PidConfig`)
2. Add a variant to `TaskConfig`
3. Implement `SimulationTask` for your new type
4. Add a constructor function and register it in the factory

This keeps the framework open for extension while maintaining a clean configuration interface.

> **Key Insight:** The Factory Pattern decouples task creation from task usage. The simulation loop doesn't know or care what kind of tasks it's running — it just calls `execute()` on each one. This is the **Open/Closed Principle** in action: the framework is open for extension (new task types) but closed for modification (the loop doesn't change).
