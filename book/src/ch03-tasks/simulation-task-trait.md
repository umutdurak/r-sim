# The SimulationTask Trait

Every task in r-sim implements the `SimulationTask` trait. This is the foundational abstraction of the framework:

```rust
#[async_trait]
pub trait SimulationTask: Send + Sync {
    async fn execute(&mut self, current_time: Duration) -> Result<(), FrameworkError>;
    fn get_inputs(&self) -> Vec<String>;
    fn get_outputs(&self) -> Vec<String>;
    fn set_input(&mut self, name: &str, value: f64);
    fn get_output_value(&self, name: &str) -> Option<f64>;
    fn get_input_value(&self, name: &str) -> Option<f64>;
    fn initialize_io(&mut self) -> Result<(), FrameworkError>;
    fn read_inputs(&mut self) -> Result<(), FrameworkError>;
    fn write_outputs(&mut self) -> Result<(), FrameworkError>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn get_name(&self) -> String;
    fn get_parameters(&self) -> HashMap<String, Parameter>;
    fn set_parameter(&mut self, name: &str, value: Parameter) -> Result<(), FrameworkError>;
}
```

Let's understand each method:

## Core Computation

### `execute(current_time)`

The heart of every task. Called once per simulation time step with the current simulation time. This is where your model's logic lives:

```rust
async fn execute(&mut self, current_time: Duration) -> Result<(), FrameworkError> {
    // Read input_var, compute output_var
    let input = self.inputs.get("sensor_reading").copied().unwrap_or(0.0);
    let output = input * self.gain + self.offset;
    self.outputs.insert("actuator_command".to_string(), output);
    Ok(())
}
```

## Data Interface

### `get_inputs()` / `get_outputs()`

Return the names of the task's input and output ports. These are used by the framework to:
- Connect tasks via dependencies
- Determine which signals to log
- Display in the web monitoring interface

### `set_input()` / `get_output_value()` / `get_input_value()`

Read and write individual signal values. The framework calls `set_input()` to pass data from upstream tasks and `get_output_value()` to read results.

## I/O Lifecycle

### `initialize_io()`

Called once at simulation startup for hardware initialization:
```rust
fn initialize_io(&mut self) -> Result<(), FrameworkError> {
    println!("Binding to UDP socket {}...", self.local_addr);
    Ok(())
}
```

### `read_inputs()` / `write_outputs()`

Called every time step, **before** and **after** `execute()` respectively:

```
read_inputs()  →  execute()  →  write_outputs()
```

This separation ensures that I/O operations happen at consistent points in the simulation loop, independent of computation logic.

## Parameters

### `get_parameters()` / `set_parameter()`

Support runtime parameter tuning via the web interface. Parameters can be `Float`, `Integer`, `Boolean`, or `String`:

```rust
fn set_parameter(&mut self, name: &str, value: Parameter) -> Result<(), FrameworkError> {
    match name {
        "gain" => {
            if let Parameter::Float(v) = value {
                self.gain = v;
                self.parameters.insert(name.to_string(), Parameter::Float(v));
            }
        }
        _ => return Err(FrameworkError::ConfigurationError(
            format!("Unknown parameter: {}", name)
        )),
    }
    Ok(())
}
```

> **Key Insight:** The trait design follows the **Read-Execute-Write** pattern common in real-time systems. This pattern prevents tasks from both reading and writing to hardware in the middle of computation, which could cause timing inconsistencies.
