# Lab 1: Basic Simulation Setup

## Objective
Install r-sim, build it from source, and run your first simulation. Understand the simulation loop, time steps, and console output.

## Background
Review: [Chapter 1](../ch01-getting-started.md), [Chapter 2](../ch02-fundamentals.md)

## Steps

### Part A: Installation (30 min)

1. Install Rust from [rustup.rs](https://rustup.rs)
2. Clone the r-sim repository:
   ```bash
   git clone https://github.com/umutdurak/r-sim.git
   cd r-sim
   ```
3. Build the project: `cargo build`
4. Verify: `./target/debug/r-sim --help`

### Part B: Running Simulations (45 min)

1. Create `lab1_basic.toml`:
   ```toml
   [[tasks]]
   type = "Custom"
   name = "Heartbeat"
   ```

2. Run for 5 seconds at 1 Hz:
   ```bash
   ./target/debug/r-sim run -c lab1_basic.toml -s 5 -t 1000
   ```
3. Count the number of "Executing Custom Task" lines. Is it what you expected?

4. Change the time step to 200ms (`-t 200`). How many executions now?

5. Add a second task:
   ```toml
   [[tasks]]
   type = "Custom"
   name = "Sensor"

   [[tasks]]
   type = "Custom"
   name = "Controller"
   ```
6. Run again. In what order do the tasks execute?

### Part C: Time Multiplier (30 min)

1. Create `lab1_slow.toml`:
   ```toml
   time_multiplier = 0.5

   [[tasks]]
   type = "Custom"
   name = "SlowTask"
   ```

2. Run: `./target/debug/r-sim run -c lab1_slow.toml -s 5 -t 1000`
3. Measure the wall-clock time with `time ./target/debug/r-sim ...`
4. How does the wall-clock time compare to a `time_multiplier = 1.0`?

### Part D: Web Monitoring (15 min)

1. Start a long simulation: `./target/debug/r-sim run -c lab1_basic.toml -s 30 -t 500`
2. In another terminal: `curl http://127.0.0.1:3030/data`
3. Note the `current_time_secs` value
4. Wait 5 seconds and query again. How much did the time advance?

## Deliverables

1. A brief report (1 page) with:
   - Number of executions for 5s at 1Hz, 5s at 5Hz, and 5s at 10Hz
   - Wall-clock time measurements for `time_multiplier` values of 0.5, 1.0, and 2.0
   - Screenshot of `/data` endpoint output
2. Your TOML configuration files

## Challenge

Calculate the theoretical vs. measured number of ticks for a given duration and time step. What causes any discrepancy?
