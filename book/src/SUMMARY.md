# Summary

[Introduction](./introduction.md)

# Part I: Foundations

- [Getting Started](./ch01-getting-started.md)
    - [Installation](./ch01-getting-started/installation.md)
    - [Your First Simulation](./ch01-getting-started/first-simulation.md)
- [Real-Time Simulation Fundamentals](./ch02-fundamentals.md)
    - [What Is Real-Time Simulation?](./ch02-fundamentals/what-is-rts.md)
    - [Time Steps and Periodic Execution](./ch02-fundamentals/time-steps.md)
    - [Real-Time vs. Non-Real-Time](./ch02-fundamentals/rt-vs-nrt.md)
    - [Determinism and Predictability](./ch02-fundamentals/determinism.md)
- [Task-Based Simulation Architecture](./ch03-tasks.md)
    - [The SimulationTask Trait](./ch03-tasks/simulation-task-trait.md)
    - [Built-in Task Types](./ch03-tasks/builtin-tasks.md)
    - [Custom Tasks](./ch03-tasks/custom-tasks.md)
    - [The TaskFactory Pattern](./ch03-tasks/task-factory.md)

# Part II: Building Simulations

- [Dependency Graphs and Execution Order](./ch04-dependencies.md)
    - [Data Flow Between Tasks](./ch04-dependencies/data-flow.md)
    - [Topological Sorting](./ch04-dependencies/topological-sort.md)
    - [Causal Loops and Algebraic Loops](./ch04-dependencies/causal-loops.md)
    - [Memory Blocks as Loop Breakers](./ch04-dependencies/memory-blocks.md)
- [I/O Subsystem and Hardware Interfaces](./ch05-io.md)
    - [GPIO: General-Purpose I/O](./ch05-io/gpio.md)
    - [Serial Communication](./ch05-io/serial.md)
    - [UDP Networking](./ch05-io/udp.md)
    - [Analog I/O and Data Acquisition](./ch05-io/analog.md)
    - [Modbus TCP for Industrial Systems](./ch05-io/modbus.md)
    - [Extending the I/O Layer](./ch05-io/extending-io.md)
- [FMU and Co-Simulation](./ch06-fmu.md)
    - [The Functional Mock-up Interface](./ch06-fmu/fmi-standard.md)
    - [Loading External Models](./ch06-fmu/loading-fmus.md)
    - [Co-Simulation Execution](./ch06-fmu/co-simulation.md)
    - [Building a Test FMU](./ch06-fmu/building-fmu.md)

# Part III: Operation and Monitoring

- [Configuration with TOML](./ch07-configuration.md)
    - [Configuration File Format](./ch07-configuration/config-format.md)
    - [Task Configuration](./ch07-configuration/task-config.md)
    - [Dependency Configuration](./ch07-configuration/dependency-config.md)
    - [Logging Configuration](./ch07-configuration/logging-config.md)
    - [Scenario Management](./ch07-configuration/scenarios.md)
- [Web Interface and Runtime Control](./ch08-web.md)
    - [Real-Time Monitoring](./ch08-web/monitoring.md)
    - [Parameter Tuning at Runtime](./ch08-web/parameter-tuning.md)
    - [Lifecycle Control](./ch08-web/lifecycle-control.md)

# Part IV: Advanced Topics

- [Testing and Validation](./ch09-testing.md)
    - [Writing Integration Tests](./ch09-testing/integration-tests.md)
    - [Test Patterns for Co-Simulation](./ch09-testing/test-patterns.md)
    - [Continuous Integration](./ch09-testing/ci.md)
- [Hardware-in-the-Loop and Beyond](./ch10-advanced.md)
    - [Software-in-the-Loop (SIL)](./ch10-advanced/sil.md)
    - [Hardware-in-the-Loop (HIL)](./ch10-advanced/hil.md)
    - [X-in-the-Loop Methodologies](./ch10-advanced/xil.md)
    - [Distributed Co-Simulation](./ch10-advanced/distributed.md)
    - [Fault Injection Testing](./ch10-advanced/fault-injection.md)

# Appendices

- [Lab Exercises](./appendix-labs.md)
    - [Lab 1: Basic Simulation Setup](./appendix-labs/lab1.md)
    - [Lab 2: Building a Custom Controller](./appendix-labs/lab2.md)
    - [Lab 3: Multi-Task Dependencies](./appendix-labs/lab3.md)
    - [Lab 4: FMU Integration](./appendix-labs/lab4.md)
    - [Lab 5: I/O and Hardware Interfacing](./appendix-labs/lab5.md)
    - [Lab 6: Full System Co-Simulation](./appendix-labs/lab6.md)
- [Reference](./appendix-reference.md)
