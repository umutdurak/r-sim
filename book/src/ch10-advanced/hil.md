# Hardware-in-the-Loop (HIL)

**Hardware-in-the-Loop** testing connects real controller hardware to a simulated plant. The simulation runs in real time, interfacing with the controller through physical I/O.

## HIL Architecture

```
┌──────────────┐    Physical I/O    ┌──────────────┐
│   r-sim      │ ◄────────────────► │  Real ECU    │
│  (Plant      │   GPIO / Serial /  │ (Controller  │
│   Model)     │   Analog / Modbus  │  Hardware)   │
└──────────────┘                    └──────────────┘
```

## Requirements for HIL

1. **Real-time execution** — The simulation must keep pace with the wall clock. Overruns mean the controller receives delayed data.
2. **Physical I/O** — The simulation must drive real electrical signals (voltage, current) to the controller's inputs.
3. **Signal conditioning** — Converting between the simulation's digital values and the physical signal ranges (e.g., 0–5V, 4–20mA).
4. **Deterministic scheduling** — For hard real-time HIL, an RTOS or PREEMPT_RT Linux kernel is needed.

## HIL with r-sim

r-sim's I/O tasks provide the foundation for HIL:

```toml
[[tasks]]
type = "Fmu"
name = "EngineModel"
path = "./target/debug/deps/libengine.dylib"

[[tasks]]
type = "Gpio"
name = "ECU_Interface"
pins = [1, 2, 3, 4, 5, 6, 7, 8]

[[tasks]]
type = "Analog"
name = "SensorOutputs"
channels = [0, 1, 2, 3]
is_input = false

[[dependencies]]
from = "EngineModel"
to = "SensorOutputs"
type = "direct"

[[dependencies]]
from = "ECU_Interface"
to = "EngineModel"
type = "direct"
```

The simulation loop:
1. Reads digital commands from the ECU via GPIO
2. Runs the engine model with those commands
3. Outputs simulated sensor values via Analog DAC
4. The ECU reads the sensor values and computes new commands

## From SIL to HIL

The transition from SIL to HIL with r-sim requires:

| Change | SIL | HIL |
|--------|-----|-----|
| Controller | Custom task (software) | Real hardware via I/O |
| Plant | FMU | FMU (same code) |
| Time step | Any (can be faster than real time) | Fixed, real-time |
| I/O | Simulated | Physical hardware |

> **Key Insight:** The plant model FMU is **the same code** in both SIL and HIL. Only the controller changes from software to hardware. This reuse is a major benefit of the simulation-based development lifecycle.
