# The Functional Mock-up Interface

The **Functional Mock-up Interface (FMI)** is an open standard developed by the Modelica Association for the exchange and co-simulation of dynamic models. Supported by over 270 tools, it is the leading standard for model-based systems engineering.

## Core Concepts

### Functional Mock-up Unit (FMU)

An **FMU** is a self-contained package that implements the FMI standard. It contains:

- **Model Description XML** — Defines the model's interface (inputs, outputs, parameters)
- **Compiled Binaries** — The simulation model compiled for one or more platforms
- **Optional Source Code** — C source for platforms without precompiled binaries

An FMU is distributed as a ZIP file with the `.fmu` extension.

### Two Simulation Modes

FMI defines two co-simulation approaches:

| Mode | Description | Solver |
|------|-------------|--------|
| **Model Exchange (ME)** | The FMU exposes its equations; the importing tool provides the solver | External |
| **Co-Simulation (CS)** | The FMU includes its own solver | Internal |

In **Model Exchange**, the importing tool calls:
```
fmi2GetDerivatives()  →  solver integrates  →  fmi2SetContinuousStates()
```

In **Co-Simulation**, the importing tool calls:
```
fmi2SetInputs()  →  fmi2DoStep(t, dt)  →  fmi2GetOutputs()
```

r-sim follows the **Co-Simulation** pattern — each FMU is told to advance one time step and reports its outputs.

## FMI Standard Versions

| Version | Year | Key Features |
|---------|------|-------------|
| FMI 1.0 | 2010 | Basic ME and CS |
| FMI 2.0 | 2014 | Improved CS, directional derivatives, event handling |
| FMI 3.0 | 2022 | Clocks, terminals, extended data types, vECU support |

## The Co-Simulation Master

In a multi-FMU setup, a **co-simulation master** orchestrates the execution:

```
┌─────────┐     ┌─────────┐     ┌─────────┐
│  FMU 1  │     │  FMU 2  │     │  FMU 3  │
│ (Engine) │     │ (Trans.) │     │ (Wheel) │
└────┬────┘     └────┬────┘     └────┬────┘
     │               │               │
     └───────────────┼───────────────┘
                     │
              ┌──────┴──────┐
              │  Co-Sim     │
              │  Master     │
              │  (r-sim)    │
              └─────────────┘
```

The master:
1. Loads all FMUs
2. Connects inputs and outputs according to the dependency graph
3. Advances all FMUs in sequence (or parallel) each time step
4. Handles data exchange between FMUs

r-sim acts as a co-simulation master, with each `FmuTask` representing one loaded FMU.

> **Key Insight:** FMI's main value is **interoperability**. An engine model from MATLAB/Simulink, a transmission model from Dymola, and a wheel model from a custom C++ tool can all be packaged as FMUs and run together in r-sim.
