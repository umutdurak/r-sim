# Time Steps and Periodic Execution

The **time step** (also called the **sample period** or **cycle time**) is the fundamental heartbeat of a real-time simulation. It defines how often the simulation loop executes.

## Fixed-Step vs. Variable-Step

There are two approaches to time advancement:

| Approach | Description | Use Case |
|----------|-------------|----------|
| **Fixed-step** | The time step is constant throughout the simulation | Real-time systems, HIL testing |
| **Variable-step** | The solver adjusts the step size based on model dynamics | Offline simulation, stiff systems |

r-sim uses **fixed-step** execution exclusively, because:

1. **Predictability** — Each loop iteration takes the same wall-clock budget
2. **Synchronization** — Hardware I/O operates on fixed sampling rates
3. **Determinism** — The same inputs always produce the same outputs at the same times

## Choosing a Time Step

The choice of time step is a critical design decision:

**Too large** (e.g., 1 second):
- Missing fast dynamics in the simulated system
- Control algorithms may become unstable
- Hardware signals are undersampled

**Too small** (e.g., 100 μs):
- Risk of overruns if computations can't finish in time
- Unnecessary CPU consumption
- I/O devices may not support such high rates

A common rule of thumb: the time step should be **5–10× smaller** than the smallest time constant in the system being simulated.

## Time Step in r-sim

In r-sim, the time step is specified in milliseconds via the CLI:

```bash
# 100ms time step = 10 Hz execution rate
./target/debug/r-sim run -c config.toml -s 10 -t 100

# 50ms time step = 20 Hz execution rate
./target/debug/r-sim run -c config.toml -s 10 -t 50

# 500ms time step = 2 Hz execution rate (good for demos)
./target/debug/r-sim run -c config.toml -s 10 -t 500
```

## The Time Multiplier

Sometimes you want the simulation to run **slower** or **faster** than real time. The `time_multiplier` configuration parameter scales the effective time step:

```toml
# Runs at half speed (good for debugging)
time_multiplier = 0.5

[[tasks]]
type = "Custom"
name = "SlowMotionTask"
```

| Multiplier | Effect |
|-----------|--------|
| 1.0 | Real-time (default) |
| 0.5 | Half speed — 100ms step takes 200ms wall clock |
| 2.0 | Double speed — 100ms step takes 50ms wall clock |

> **Exercise:** Create a configuration with `time_multiplier = 0.5` and run it with `-t 500`. Observe how the simulation takes 6 real seconds for a 3-second simulation (`-s 3`).

## Simulation Time vs. Wall Clock Time

It's important to distinguish between:

- **Simulation time** — The time coordinate in the simulated world
- **Wall clock time** — The real-world elapsed time

With `time_multiplier = 1.0`, these are identical. With other values, they diverge. r-sim's output shows the simulation time:

```
run_framework: Simulation Time: 500ms
run_framework: Simulation Time: 1s
run_framework: Simulation Time: 1.500s
```

The `/data` web endpoint reports `current_time_secs`, which is the simulation time.
