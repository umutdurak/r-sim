# Lab 6: Full System Co-Simulation

## Objective
Design and implement a complete co-simulation integrating all r-sim features: FMU models, multiple I/O types, dependency management, CSV logging, and web monitoring.

## Background
Review: All chapters (Ch 1–9)

## Scenario: Thermal Control System

You are designing a thermal management system with the following components:

| Component | r-sim Task Type | Role |
|-----------|----------------|------|
| Temperature Sensor | Analog (input) | Reads room temperature |
| Thermostat Controller | Custom | PID-like control logic |
| Heater Model | FMU | Simulates thermal dynamics |
| Display Interface | Serial | Shows current temp on LCD |
| Network Logger | UDP | Sends data to monitoring station |

## Steps

### Part A: Configuration Design (45 min)

Design the TOML configuration file. Consider:
- Which tasks need I/O phases?
- What are the dependencies? (Sensor → Controller → Heater, Heater → Sensor via memory block)
- What signals should be logged to CSV?

### Part B: Implementation (90 min)

1. Create `lab6_thermal.toml` with all 5 tasks:
   ```toml
   time_multiplier = 1.0

   [[tasks]]
   type = "Analog"
   name = "TemperatureSensor"
   channels = [0]
   is_input = true

   [[tasks]]
   type = "Custom"
   name = "ThermostatController"

   [[tasks]]
   type = "Fmu"
   name = "HeaterModel"
   path = "./target/debug/deps/libfmu_test.dylib"

   [[tasks]]
   type = "Serial"
   name = "DisplayModule"
   port = "/dev/ttyUSB0"
   baud_rate = 9600

   [[tasks]]
   type = "Udp"
   name = "NetworkLogger"
   local_addr = "127.0.0.1:18080"
   remote_addr = "127.0.0.1:18081"

   [[dependencies]]
   from = "TemperatureSensor"
   to = "ThermostatController"
   type = "direct"
   data_flow = "output_to_input"

   [[dependencies]]
   from = "ThermostatController"
   to = "HeaterModel"
   type = "direct"
   data_flow = "output_to_input"

   [[dependencies]]
   from = "HeaterModel"
   to = "TemperatureSensor"
   type = "memory_block"
   data_flow = "output_to_input"

   [[dependencies]]
   from = "ThermostatController"
   to = "DisplayModule"
   type = "direct"
   data_flow = "output_to_input"

   [[dependencies]]
   from = "ThermostatController"
   to = "NetworkLogger"
   type = "direct"
   data_flow = "output_to_input"

   [logging]
   log_file = "thermal_simulation.csv"
   log_interval_millis = 100
   logged_outputs = { ThermostatController = ["custom_output"] }
   ```

2. Run: `./target/debug/r-sim run -c lab6_thermal.toml -s 30 -t 100`

### Part C: Monitoring and Tuning (45 min)

1. While the simulation runs, query all web endpoints
2. Use `/graph` to visualize the task dependency structure
3. Tune the HeaterModel's `gain` parameter via `/parameters/set`
4. Pause the simulation, inspect state, then resume
5. Stop the simulation gracefully

### Part D: Analysis (60 min)

1. Open the CSV log file in a spreadsheet or Python
2. Plot the controller output over time
3. Identify the steady-state behavior
4. Run the test suite to verify nothing is broken:
   ```bash
   cargo test --test integration_test -- --test-threads=1
   ```

## Deliverables

1. Complete `lab6_thermal.toml` configuration
2. Console output showing all 5 tasks executing in dependency order
3. Web API outputs: `/data`, `/graph`, `/parameters`  
4. CSV log file with plotted data
5. A 2-page report discussing:
   - The execution order and why it's correct
   - The effect of the memory block on system behavior
   - What would change if this were a HIL setup with real hardware

## Challenge

Design a fault injection scenario: at t=15s, the temperature sensor "fails" (its gain becomes 0). How would the system respond? Implement this using runtime parameter tuning.
