# Data Flow Between Tasks

In a multi-task simulation, data flows from one task's outputs to another task's inputs. These connections define the **data flow graph**.

## Defining Dependencies

In r-sim, dependencies are declared in the TOML configuration:

```toml
[[tasks]]
type = "Custom"
name = "Sensor"

[[tasks]]
type = "Custom"
name = "Controller"

[[tasks]]
type = "Custom"
name = "Actuator"

[[dependencies]]
from = "Sensor"
to = "Controller"
type = "direct"
data_flow = "output_to_input"

[[dependencies]]
from = "Controller"
to = "Actuator"
type = "direct"
data_flow = "output_to_input"
```

This defines a simple chain: `Sensor → Controller → Actuator`.

## Dependency Types

r-sim supports two dependency types:

### Direct Dependencies

A **direct dependency** means "from must execute before to" in the same time step. The output of `from` is immediately available as the input of `to`.

```toml
[[dependencies]]
from = "Producer"
to = "Consumer"
type = "direct"
data_flow = "output_to_input"
```

### Memory Block Dependencies

A **memory block dependency** introduces a one-step delay. The output of `from` at time step `n` becomes available as the input of `to` at time step `n+1`. This is used to break causal loops (covered in a later section).

```toml
[[dependencies]]
from = "TaskA"
to = "TaskB"
type = "memory_block"
data_flow = "output_to_input"
```

## The Data Flow Model

After the framework resolves dependencies, it copies data between tasks between execution phases:

```
Time Step n:
  1. Read I/O for all tasks
  2. Execute tasks in topological order:
     a. Execute Sensor      → produces output_value
     b. Copy Sensor.output → Controller.input
     c. Execute Controller  → produces control_signal
     d. Copy Controller.output → Actuator.input
     e. Execute Actuator    → uses control_signal
  3. Write I/O for all tasks
  4. Log data
```

> **Key Insight:** Data flows are resolved at the framework level, not by the tasks themselves. Individual tasks only know about their own inputs and outputs — they don't "reach out" to other tasks. This loose coupling makes tasks reusable and independently testable.

Try it:

```bash
./target/debug/r-sim run -c dependent_tasks.toml -s 2 -t 500
```

You should see tasks executing in dependency order, with the output confirming "Producer" runs before "Consumer".
