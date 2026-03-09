# Memory Blocks as Loop Breakers

The standard technique for resolving causal loops in discrete-time simulation is to insert a **memory block** (also called a **unit delay** or **Z⁻¹ block**) into the feedback path. This introduces a one-step delay that breaks the algebraic loop while preserving the physical feedback semantics.

## How It Works

Instead of requiring the Plant's output to be available *instantaneously* for the Controller in the same time step, the memory block says: "The Controller will use the Plant's output from the *previous* time step."

```
Time Step n:  Controller gets Plant output from step n-1
              Controller computes command
              Plant gets Controller command (current step)
              Plant computes next state
              Plant output is stored for step n+1
```

This is mathematically equivalent to a discrete-time delay:

```
u[n] = g(y[n-1])    ← Controller uses delayed plant output
y[n] = f(u[n])      ← Plant uses current controller command
```

## Configuration in r-sim

To break a cycle, change one of the dependencies to type `"memory_block"`:

```toml
[[tasks]]
type = "Custom"
name = "Controller"

[[tasks]]
type = "Custom"
name = "Plant"

[[dependencies]]
from = "Controller"
to = "Plant"
type = "direct"
data_flow = "output_to_input"

[[dependencies]]
from = "Plant"
to = "Controller"
type = "memory_block"           # ← One-step delay breaks the cycle
data_flow = "output_to_input"
```

Now the dependency graph is acyclic:
- Direct edge: Controller → Plant
- Memory block edge: Plant → Controller (ignored in topological sort)

The topological sort succeeds: **[Controller, Plant]**

Try it:

```bash
./target/debug/r-sim run -c tests/test_configs/causal_loop_memory_block.toml -s 2 -t 500
```

You'll see both tasks executing without the "cycle detected" error.

## Where to Place the Memory Block

A natural question is: which edge in the cycle should become the memory block? The answer depends on physics:

1. **Feedback path** — Usually the sensor reading going back to the controller is delayed (the controller uses the latest available measurement, which is inherently from a previous sampling instant)
2. **Feedforward path** — Usually kept direct (the controller's command should be applied to the plant as quickly as possible)

In most control systems, the feedback path is the natural location for the memory block, because physical sensors already introduce a measurement delay.

## Impact on Accuracy

The memory block introduces a delay of one time step. If your time step is:
- **10ms** — the delay is 10ms, usually negligible for systems with time constants > 100ms
- **1s** — the delay is 1s, which could significantly affect system dynamics

**Rule of thumb:** The time step should be at least 5–10× smaller than the system's smallest time constant to keep the memory block delay negligible.

> **Exercise:** Create a configuration with three tasks A, B, C forming a loop: A → B → C → A. Insert a memory block to break the cycle. How many different placements are possible? Do they all produce the same execution order?
