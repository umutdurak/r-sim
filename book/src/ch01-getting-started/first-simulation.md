# Your First Simulation

Let's run a minimal simulation to see r-sim in action. Create a file called `hello_sim.toml`:

```toml
# hello_sim.toml — A minimal r-sim configuration

[[tasks]]
type = "Custom"
name = "HelloTask"
```

This configuration defines a single **Custom** task named "HelloTask". Now run it:

```bash
./target/debug/r-sim run -c hello_sim.toml -s 3 -t 1000
```

The command-line arguments are:

| Flag | Meaning |
|------|---------|
| `-c hello_sim.toml` | Path to the configuration file |
| `-s 3` | Simulation duration: 3 seconds |
| `-t 1000` | Time step: 1000 milliseconds (1 second) |

You should see output like:

```
r-sim: Real-Time Co-Simulation Framework
Simulation Duration: 3s
Time Step: 1000ms
Config File: hello_sim.toml
...
run_framework: Creating task: Custom(CustomConfig { name: "HelloTask" })
run_framework: Execution order determined.
run_framework: Tick received.
run_framework: Current status: Running
run_framework: Simulation Time: 1s
run_framework: Executing task: HelloTask
  Executing Custom Task HelloTask: 1s
...
run_framework: Simulation finished.
```

## What Just Happened?

Let's break down the simulation lifecycle:

1. **Configuration parsing** — r-sim reads `hello_sim.toml` and deserializes it into a `SimulationConfig` struct
2. **Task creation** — The `TaskFactory` creates a `CustomTask` instance from the config
3. **Graph construction** — A dependency graph is built (trivial in this case — one node, no edges)
4. **Topological sorting** — The execution order is determined (just "HelloTask")
5. **Simulation loop** — On each tick (every 1000ms):
   - I/O read phase (for I/O tasks)
   - Task execution phase (runs `HelloTask.execute()`)
   - I/O write phase (for I/O tasks)
   - Logging phase
6. **Shutdown** — After 3 seconds, the simulation stops and the web server shuts down gracefully

## Adding a Second Task

Let's make it more interesting with two tasks:

```toml
# two_tasks.toml

[[tasks]]
type = "Custom"
name = "SensorReader"

[[tasks]]
type = "Custom"
name = "Controller"
```

```bash
./target/debug/r-sim run -c two_tasks.toml -s 2 -t 500
```

Now you'll see both tasks executing every 500ms. Since there are no dependencies defined, r-sim treats them as **independent** tasks that can conceptually execute in parallel.

## Monitoring While Running

While the simulation runs, open a browser or use `curl` to query the web monitoring interface:

```bash
# In another terminal:
curl http://127.0.0.1:3030/data
```

You'll receive a JSON response with the current simulation state:

```json
{
  "current_time_secs": 1.5,
  "task_execution_times_micros": { "SensorReader": 42, "Controller": 38 }
}
```

This is the `/data` endpoint — we'll explore the full web API in [Chapter 8](../ch08-web.md).

> **Exercise:** Try changing the time step to 100ms (`-t 100`) and observe how the simulation runs at a faster rate. How does this affect the number of ticks in a 2-second simulation?
