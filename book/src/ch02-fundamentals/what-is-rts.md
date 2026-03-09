# What Is Real-Time Simulation?

A **real-time simulation** is a simulation in which the passage of time in the simulated world corresponds to the passage of time in the real world. If one second passes on the wall clock, one second passes in the simulation.

This seemingly simple definition has profound implications.

## The Wall Clock Constraint

In a non-real-time simulation (like a finite-element analysis or a Monte Carlo study), the simulator runs as fast as it can. A simulation of 10 seconds of physical behavior might complete in 1 second or 1 hour — it doesn't matter, because the results are the same either way.

In a real-time simulation, **timing is a correctness requirement**. Consider:

- An engine controller simulation that must read sensor values and compute actuator commands every 10ms
- A flight simulator that must update the visual display at 60 Hz
- A power grid protection system that must detect a fault within 5ms

If the computation for a single time step takes longer than the time step itself, the simulation **fails to keep up** with reality. This is called an **overrun**, and in safety-critical systems, it can have serious consequences.

## Applications

Real-time simulation is used extensively across engineering disciplines:

| Domain | Application | Typical Time Step |
|--------|------------|-------------------|
| Automotive | Engine ECU testing (HIL) | 1–10 ms |
| Aerospace | Flight dynamics simulation | 10–50 ms |
| Power Systems | Grid protection testing | 50–500 μs |
| Robotics | Motion control | 1–5 ms |
| Industrial Automation | PLC testing | 10–100 ms |

## The Simulation Loop

At the heart of every real-time simulation is a **fixed-step simulation loop**:

```
loop {
    wait_for_next_tick()      // Block until the next time step
    read_inputs()             // Sample sensors / receive data
    execute_tasks()           // Run all simulation models
    write_outputs()           // Drive actuators / send data
    log_data()                // Record results
}
```

In r-sim, this loop is implemented using Tokio's `interval::tick()` mechanism, which provides a periodic timer that fires at precisely the configured time step.

```toml
# The time step is configured in milliseconds
# This runs the loop at 100ms intervals (10 Hz)
./target/debug/r-sim run -c config.toml -s 10 -t 100
```

> **Key Insight:** The time step must be large enough for all tasks to complete their computations within it. If tasks take 8ms total and the time step is 10ms, you have 2ms of **slack** — margin for jitter and overhead.
