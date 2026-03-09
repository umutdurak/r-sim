# Custom Tasks

The `Custom` task type is the simplest and most flexible task in r-sim. To understand how to build your own simulation models, let's examine how `CustomTask` is implemented and then discuss how you would create your own task type.

## The CustomTask Implementation

```rust
pub struct CustomTask {
    name: String,
    inputs: HashMap<String, f64>,
    outputs: HashMap<String, f64>,
    parameters: HashMap<String, Parameter>,
}

#[async_trait]
impl SimulationTask for CustomTask {
    async fn execute(&mut self, current_time: Duration) -> Result<(), FrameworkError> {
        println!("  Executing Custom Task {}: {:?}", self.name, current_time);
        // Simple logic: increment the output value
        if let Some(output_val) = self.outputs.get_mut("custom_output") {
            *output_val += 1.0;
        }
        Ok(())
    }
    
    fn get_inputs(&self) -> Vec<String> {
        self.inputs.keys().cloned().collect()
    }
    
    fn get_outputs(&self) -> Vec<String> {
        self.outputs.keys().cloned().collect()
    }
    
    // ... other trait methods
}
```

## Designing Your Own Task

To create a custom simulation model, you would:

1. **Define the struct** with your model's state variables
2. **Implement the constructor** initializing inputs, outputs, and parameters
3. **Implement `execute()`** with your model's computation
4. **Register it** in the `TaskFactory`

### Example: A PID Controller

Here's how you might conceptually design a PID controller task:

```rust
pub struct PidControllerTask {
    name: String,
    inputs: HashMap<String, f64>,   // "setpoint", "process_value"
    outputs: HashMap<String, f64>,  // "control_output"
    parameters: HashMap<String, Parameter>,
    
    // PID state
    kp: f64,          // Proportional gain
    ki: f64,          // Integral gain  
    kd: f64,          // Derivative gain
    integral: f64,    // Accumulated integral
    prev_error: f64,  // Previous error for derivative
}
```

The `execute()` method would compute:

```rust
async fn execute(&mut self, current_time: Duration) -> Result<(), FrameworkError> {
    let setpoint = self.inputs.get("setpoint").copied().unwrap_or(0.0);
    let pv = self.inputs.get("process_value").copied().unwrap_or(0.0);
    
    let error = setpoint - pv;
    let dt = 0.1; // Time step in seconds
    
    self.integral += error * dt;
    let derivative = (error - self.prev_error) / dt;
    
    let output = self.kp * error + self.ki * self.integral + self.kd * derivative;
    
    self.outputs.insert("control_output".to_string(), output);
    self.prev_error = error;
    
    Ok(())
}
```

This pattern — reading inputs, computing, writing outputs — is the fundamental rhythm of every simulation task.

> **Exercise:** Sketch out the struct and `execute()` method for a first-order low-pass filter task. It should read `input_signal`, apply the filter equation `y[n] = α * x[n] + (1 - α) * y[n-1]`, and write `filtered_output`.
