# Software-in-the-Loop (SIL)

**Software-in-the-Loop** testing validates the actual production software against a simulated plant model, without any physical hardware. The control algorithm runs as compiled code (not as a model), while the system it controls is simulated.

## SIL Architecture

```
┌────────────────────────────────┐
│         r-sim Framework        │
│                                │
│  ┌──────────┐  ┌────────────┐  │
│  │Production│  │   Plant     │  │
│  │ Control  │→ │   Model     │  │
│  │ Software │← │   (FMU)     │  │
│  └──────────┘  └────────────┘  │
│                                │
│  Data exchange via framework   │
└────────────────────────────────┘
```

## Benefits

- **No hardware needed** — Test purely in software
- **Fast iteration** — Recompile and rerun in seconds
- **Full coverage** — Test edge cases impossible to reproduce with hardware
- **CI integration** — Run SIL tests as part of your build pipeline

## SIL with r-sim

To implement SIL testing:

1. Compile your production control code as a Custom task or FMU
2. Create an FMU plant model simulating the physical system
3. Connect them via dependencies
4. Run the simulation and verify the control algorithm's behavior

```toml
[[tasks]]
type = "Custom"
name = "ProductionController"    # Your actual control code

[[tasks]]
type = "Fmu"
name = "PlantSimulation"
path = "./target/debug/deps/libplant_model.dylib"

[[dependencies]]
from = "ProductionController"
to = "PlantSimulation"
type = "direct"

[[dependencies]]
from = "PlantSimulation"
to = "ProductionController"
type = "memory_block"
```

> **Key Insight:** In SIL, the control software runs at a simulated clock rate, not the CPU's maximum speed. The `time_multiplier` can be used to run faster than real time for batch testing, or slower for debugging.
