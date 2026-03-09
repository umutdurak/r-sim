# Determinism and Predictability

**Determinism** means that given the same inputs and initial conditions, the simulation produces **exactly the same outputs** every time it runs. This property is crucial for:

- **Reproducibility** — Debugging and regression testing require repeatable behavior
- **Certification** — Safety-critical systems must demonstrate predictable behavior
- **Validation** — Comparing expected vs. actual results is only meaningful if the simulation is deterministic

## Sources of Non-Determinism

Several factors can introduce non-determinism into a simulation:

### 1. Floating-Point Arithmetic

On different hardware or with different compiler settings, floating-point operations may produce slightly different results:

```rust
// These may not be equal across platforms:
let a = 0.1 + 0.2;  // May be 0.30000000000000004
let b = 0.3;         // May be 0.29999999999999998
```

**Mitigation:** Use consistent FPU settings, avoid reordering of operations, or use fixed-point arithmetic for critical paths.

### 2. Thread Scheduling

If tasks run on multiple threads, the operating system's scheduler determines their execution order, which may vary between runs.

**Mitigation:** r-sim uses **topological sorting** to determine a fixed execution order, and executes tasks sequentially within the simulation loop. This eliminates scheduling-related non-determinism.

### 3. Non-Deterministic Inputs

Real hardware produces inherently non-deterministic data (sensor noise, network latency, etc.).

**Mitigation:** For testing, use simulated I/O (as r-sim does by default). For replay-based debugging, record and replay input streams.

### 4. Random Number Generation

If your models use random numbers, different seeds produce different results.

**Mitigation:** Use deterministic pseudo-random number generators (PRNGs) with fixed seeds.

## Determinism in r-sim

r-sim achieves determinism through several design choices:

1. **Fixed execution order** — Tasks are topologically sorted and executed in a deterministic sequence
2. **Fixed time step** — No adaptive stepping that could change based on runtime conditions
3. **Sequential execution** — Tasks execute one at a time within the simulation loop, eliminating thread scheduling variability
4. **Simulated I/O** — Built-in I/O tasks are simulated, providing repeatable behavior

> **Note:** Full hardware-level determinism (bit-exact reproducibility across platforms) is beyond r-sim's current scope but is an active area of development (`REQ-DETERMINISTIC-EXECUTION`).

> **Exercise:** Run the same configuration twice with r-sim. Compare the output logs. Which parts are deterministic? Which parts vary (hint: look at execution times)?
