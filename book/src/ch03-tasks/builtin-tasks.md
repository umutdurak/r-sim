# Built-in Task Types

r-sim ships with several task types that cover common simulation building blocks. Each represents a different kind of component you might find in a real-time system.

## CustomTask

The general-purpose task for arbitrary computation logic.

```toml
[[tasks]]
type = "Custom"
name = "MyController"
```

A `CustomTask` starts with one output (`custom_output`) initialized to `0.0`. Its `execute()` method increments this value, simulating a simple counter. In practice, you would extend this with your own computation logic.

**Use for:** Controllers, signal processing, data transformation, any computation that doesn't involve specific I/O hardware.

## FmuTask

Loads and executes an external model compiled as a dynamic library (`.dylib`, `.so`, or `.dll`).

```toml
[[tasks]]
type = "Fmu"
name = "PlantModel"
path = "./target/debug/deps/libfmu_test.dylib"
```

The `FmuTask` loads the library at startup, finds the `do_step` C function, and calls it with the current simulation time each time step. We'll explore this in depth in [Chapter 6](../ch06-fmu.md).

**Use for:** External physics models, plant simulations, legacy models.

## GpioTask

Simulates GPIO (General-Purpose Input/Output) pins for digital I/O.

```toml
[[tasks]]
type = "Gpio"
name = "DigitalSensors"
pins = [1, 2, 3, 4]
```

The task reads from and writes to the specified pins each time step.

**Use for:** Digital sensors, switches, LEDs, relay control.

## SerialTask

Simulates a serial communication port.

```toml
[[tasks]]
type = "Serial"
name = "RS232_Link"
port = "/dev/ttyUSB0"
baud_rate = 115200
```

**Use for:** UART devices, sensor modules, embedded system communication.

## UdpTask

Simulates UDP network communication.

```toml
[[tasks]]
type = "Udp"
name = "NetworkComm"
local_addr = "127.0.0.1:8080"
remote_addr = "127.0.0.1:8081"
```

**Use for:** Network-based data exchange, distributed system interfaces, remote sensor data.

## AnalogTask

Simulates analog I/O with optional high-speed data acquisition.

```toml
[[tasks]]
type = "Analog"
name = "ADC_Input"
channels = [0, 1, 2, 3]
is_input = true
sampling_rate_hz = 10000
```

**Use for:** Analog sensors (temperature, pressure, voltage), DAC outputs, data acquisition systems.

## ModbusTcpTask

Simulates a Modbus TCP industrial communication endpoint.

```toml
[[tasks]]
type = "ModbusTcp"
name = "PLC_Interface"
ip_address = "192.168.1.100"
port = 502
```

**Use for:** PLC communication, industrial automation, SCADA systems.

## Comparison Table

| Task Type | Config Fields | I/O Phase | Typical Domain |
|-----------|--------------|-----------|----------------|
| Custom | `name` | No | Computation |
| Fmu | `name`, `path` | No | Model integration |
| Gpio | `name`, `pins` | Yes | Digital I/O |
| Serial | `name`, `port`, `baud_rate` | Yes | Serial comm |
| Udp | `name`, `local_addr`, `remote_addr` | Yes | Networking |
| Analog | `name`, `channels`, `is_input`, `sampling_rate_hz` | Yes | Data acquisition |
| ModbusTcp | `name`, `ip_address`, `port` | Yes | Industrial |

> **Note:** Tasks with I/O phases (GPIO, Serial, UDP, Analog, ModbusTcp) have their `read_inputs()` and `write_outputs()` methods called each time step. Tasks without I/O (Custom, FMU) only have `execute()` called.
