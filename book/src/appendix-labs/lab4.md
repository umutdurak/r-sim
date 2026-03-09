# Lab 4: FMU Integration

## Objective
Build, load, and execute an external model (FMU) alongside other r-sim tasks. Monitor FMU outputs via the web API.

## Background
Review: [Chapter 6](../ch06-fmu.md)

## Steps

### Part A: Building the FMU (45 min)

1. Examine `fmu_test/src/lib.rs`:
   ```bash
   cat fmu_test/src/lib.rs
   ```
2. Build the project: `cargo build`
3. Verify the library exists: `ls target/debug/deps/libfmu_test.dylib`

### Part B: Loading the FMU (30 min)

1. Create `lab4_fmu.toml`:
   ```toml
   [[tasks]]
   type = "Fmu"
   name = "PhysicsModel"
   path = "./target/debug/deps/libfmu_test.dylib"
   ```
2. Run: `./target/debug/r-sim run -c lab4_fmu.toml -s 5 -t 500`
3. Locate the "Library loaded successfully" message
4. Observe the FMU results at each time step

### Part C: FMU + Controller (45 min)

1. Add a controller task and dependency:
   ```toml
   [[tasks]]
   type = "Custom"
   name = "Controller"

   [[dependencies]]
   from = "PhysicsModel"
   to = "Controller"
   type = "direct"
   data_flow = "output_to_input"
   ```
2. Run and verify both tasks execute
3. Use `/parameters` to inspect the FMU's parameters

### Part D: Runtime Parameter Tuning (30 min)

1. Start a long simulation
2. Query FMU parameters
3. Change the `gain` parameter using the web API
4. Observe the effect on FMU outputs via `/data`

## Deliverables

1. Console output proving the FMU loaded and executed
2. `/parameters` output before and after tuning
3. Brief report: What does the `do_step` function compute? If you modified it, what equation would you implement?

## Challenge

Modify `fmu_test/src/lib.rs` to implement a simple first-order filter: `y[n] = 0.8 * y[n-1] + 0.2 * t`. Rebuild and test.
