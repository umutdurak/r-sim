# Co-Simulation Execution

When multiple FMUs and other tasks are combined in a single simulation, they form a **co-simulation**. This section explains how r-sim orchestrates the execution of heterogeneous tasks.

## The Co-Simulation Loop

```
For each time step:
  1. Read I/O for all I/O tasks
  2. For each task (in topological order):
     a. Copy upstream outputs → current task inputs
     b. Execute the task (FMU: call do_step, Custom: run logic)
     c. Store outputs for downstream tasks
  3. Write I/O for all I/O tasks
  4. Log data to CSV
```

## Multi-Model Example

Consider a vehicle simulation with three components:

```toml
[[tasks]]
type = "Fmu"
name = "EngineModel"
path = "./target/debug/deps/libengine.dylib"

[[tasks]]
type = "Custom"
name = "TransmissionController"

[[tasks]]
type = "Fmu"
name = "WheelDynamics"
path = "./target/debug/deps/libwheel.dylib"

[[dependencies]]
from = "EngineModel"
to = "TransmissionController"
type = "direct"

[[dependencies]]
from = "TransmissionController"
to = "WheelDynamics"
type = "direct"
```

This creates a chain: `EngineModel → TransmissionController → WheelDynamics`

Each time step:
1. EngineModel runs `do_step(t)`, producing torque output
2. TransmissionController reads engine torque, computes gear ratio, writes wheel command
3. WheelDynamics runs `do_step(t)`, using the transmission command

## Data Exchange

Data flows between tasks via the `inputs`/`outputs` HashMaps. The framework copies values between connected tasks according to the defined dependencies:

```
EngineModel.outputs["output_var"]  →  TransmissionController.inputs["input_var"]
```

## Synchronization Considerations

In real-world co-simulation, synchronization between models is critical:

- **Communication step size** — How often data is exchanged between models (= the simulation time step in r-sim)
- **Extrapolation** — Between communication points, models may extrapolate inputs
- **Stability** — Large step sizes with tightly coupled models can cause numerical instability

r-sim uses a **fixed communication step** equal to the simulation time step, with no extrapolation. This is the simplest and most predictable approach, suitable for most practical applications.

> **Key Insight:** The beauty of co-simulation is that each model can be developed, tested, and validated independently. They only need to agree on the interface (inputs, outputs) and the communication step size.
