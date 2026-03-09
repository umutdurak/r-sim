# Real-Time Co-Simulation with r-sim

Welcome to *Real-Time Co-Simulation with r-sim*, a comprehensive guide to understanding and practicing real-time simulation concepts using the r-sim framework.

## Who Is This Book For?

This book is designed for **graduate students** in computer science and engineering who are studying real-time systems, co-simulation, and hardware/software-in-the-loop testing. It assumes familiarity with:

- Basic programming concepts (ideally some Rust experience)
- Undergraduate-level control systems or systems engineering
- Command-line tools and TOML/JSON configuration formats

## What You Will Learn

By the end of this book, you will:

1. **Understand** the fundamental principles of real-time simulation — time steps, periodicity, determinism, and synchronization
2. **Build** simulation systems from modular, reusable tasks connected through dependency graphs
3. **Integrate** external models via the Functional Mock-up Interface (FMU) standard
4. **Interface** with hardware through GPIO, Serial, UDP, Analog, and Modbus protocols
5. **Monitor** simulations in real time using web-based dashboards
6. **Test** your simulations systematically with integration tests and validation patterns
7. **Apply** these concepts to HIL, SIL, and distributed co-simulation scenarios

## How This Book Is Organized

The book is divided into four parts:

- **Part I: Foundations** — Installation, first simulation, and the theoretical foundations of real-time simulation
- **Part II: Building Simulations** — Dependency graphs, I/O hardware interfaces, and FMU co-simulation
- **Part III: Operation and Monitoring** — TOML configuration, web monitoring, and runtime control
- **Part IV: Advanced Topics** — Testing, HIL/SIL methodologies, and distributed simulation

Each chapter includes **running examples** that you can execute with r-sim, and **lab exercises** in the appendix provide structured assignments for hands-on practice.

## About r-sim

r-sim is a real-time co-simulation framework written in Rust. It provides:

- A task-based simulation engine with configurable time steps
- Topological dependency ordering with causal loop detection
- Multi-protocol I/O (GPIO, Serial, UDP, Analog, Modbus TCP)
- FMU integration for external model loading
- A web-based monitoring and control interface
- Declarative TOML configuration with scenario management

The source code is available at [github.com/umutdurak/r-sim](https://github.com/umutdurak/r-sim) under the MIT license.

## Conventions Used

Throughout this book:

- `monospace` text indicates code, commands, file names, or configuration keys
- Code blocks with a file path comment (e.g., `// config.toml`) show configuration examples you can save and run
- **Bold** terms indicate concepts defined for the first time
- > **Note:** blocks provide additional context or tips
- > **Exercise:** blocks suggest activities to deepen understanding

Let's begin!
