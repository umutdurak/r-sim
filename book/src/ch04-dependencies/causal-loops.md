# Causal Loops and Algebraic Loops

In many real-world systems, feedback is a fundamental mechanism. A thermostat reads the room temperature and adjusts the heater, which in turn affects the room temperature. This creates a **feedback loop**. When translated to a simulation, feedback creates **causal loops** — cyclic dependencies between tasks.

## Understanding the Problem

Consider a controller and a plant model:

```
Controller → Plant → Controller
```

The Controller needs the Plant's output to compute its command, and the Plant needs the Controller's output to compute the next state. This is a **causal loop**: a directed cycle in the dependency graph.

### Why Cycles Are Problematic

If both tasks have **direct** dependencies on each other, neither can execute first — they are locked in a circular dependency. Topological sort fails because there is no valid linear ordering of a cycle.

In continuous mathematics, this corresponds to an **algebraic loop**: a set of simultaneous equations like:

```
y = f(u)
u = g(y)
```

These can be solved by iteration (Newton's method, fixed-point iteration), but in a discrete-time simulation with fixed time steps, we need a different approach.

## Detecting Cycles in r-sim

When you define a cyclic dependency:

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

[[dependencies]]
from = "Plant"
to = "Controller"
type = "direct"      # ← This creates a cycle!
```

r-sim will detect it and report:

```
Graph error: Cycle detected. Cannot perform topological sort.
```

The simulation will not start until the cycle is resolved.

> **Key Insight:** Cycles are not always errors — they often represent legitimate physical feedback. The question is how to implement them correctly in a discrete-time simulation.
