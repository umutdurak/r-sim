# Lab 3: Multi-Task Dependencies

## Objective
Create complex dependency graphs, understand topological sorting, and resolve causal loops.

## Background
Review: [Chapter 4](../ch04-dependencies.md)

## Steps

### Part A: Linear Chain (30 min)

Create a 4-task chain: `Sensor → Filter → Controller → Actuator`

```toml
[[tasks]]
type = "Custom"
name = "Sensor"

[[tasks]]
type = "Custom"
name = "Filter"

[[tasks]]
type = "Custom"
name = "Controller"

[[tasks]]
type = "Custom"
name = "Actuator"

[[dependencies]]
from = "Sensor"
to = "Filter"
type = "direct"
data_flow = "output_to_input"

[[dependencies]]
from = "Filter"
to = "Controller"
type = "direct"
data_flow = "output_to_input"

[[dependencies]]
from = "Controller"
to = "Actuator"
type = "direct"
data_flow = "output_to_input"
```

Verify the execution order from the console output.

### Part B: Diamond Graph (30 min)

Create a diamond: `Sensor → Filter, Sensor → Estimator, Filter → Controller, Estimator → Controller`

Predict the valid execution orders. Run and verify.

### Part C: Causal Loop (30 min)

1. Create a cycle: `Controller → Plant → Controller` (both direct)
2. Run and observe the error message
3. Break the cycle with a memory block on the feedback path
4. Run again and verify success

### Part D: Triple Loop (30 min)

Create three tasks A, B, C in a ring: `A → B → C → A`. Break the cycle and verify.

## Deliverables

1. Console output for each topology
2. List of valid topological orderings for the diamond graph (determine manually, verify with r-sim)
3. Explanation of where you placed the memory block and why

## Challenge

What is the maximum number of valid topological orderings for N independent tasks?
