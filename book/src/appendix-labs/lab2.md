# Lab 2: Building a Custom Controller

## Objective
Understand the `SimulationTask` trait by examining how Custom tasks work. Create a configuration that simulates a controller-plant interaction.

## Background
Review: [Chapter 3](../ch03-tasks.md)

## Steps

### Part A: The Custom Task (45 min)

1. Create `lab2_controller.toml`:
   ```toml
   [[tasks]]
   type = "Custom"
   name = "TemperatureController"

   [[tasks]]
   type = "Custom"
   name = "HeaterPlant"

   [[dependencies]]
   from = "TemperatureController"
   to = "HeaterPlant"
   type = "direct"
   data_flow = "output_to_input"
   ```

2. Run: `./target/debug/r-sim run -c lab2_controller.toml -s 5 -t 500`
3. Observe the execution order — the controller runs before the plant

### Part B: Parameter Tuning (45 min)

1. Start a long simulation: `./target/debug/r-sim run -c lab2_controller.toml -s 60 -t 500`
2. Query parameters: `curl http://127.0.0.1:3030/parameters`
3. Change the HeaterPlant's gain:
   ```bash
   curl -X POST http://127.0.0.1:3030/parameters/set \
     -H "Content-Type: application/json" \
     -d '{"task_name": "HeaterPlant", "param_name": "gain", "param_value": {"Float": 5.0}}'
   ```
4. Query parameters again to verify the change
5. Experiment with different gain values

### Part C: Adding Feedback (60 min)

1. Add a feedback dependency with a memory block:
   ```toml
   [[dependencies]]
   from = "HeaterPlant"
   to = "TemperatureController"
   type = "memory_block"
   data_flow = "output_to_input"
   ```
2. Run the simulation again
3. Verify that both tasks still execute (no cycle error)
4. Use `/graph` to view the dependency structure

## Deliverables

1. Report containing:
   - Execution order with and without feedback
   - Parameter values before and after tuning (from curl output)
   - `/graph` JSON output showing the dependency structure
2. All TOML configuration files

## Challenge

Why can't we use `type = "direct"` for both dependencies? What would happen mathematically if we could solve the feedback instantaneously?
