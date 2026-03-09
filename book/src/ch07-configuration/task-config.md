# Task Configuration

Each task is defined as a `[[tasks]]` entry with at minimum a `type` and `name`:

```toml
[[tasks]]
type = "Custom"
name = "MyTask"
```

## Config Fields by Task Type

### Custom
```toml
[[tasks]]
type = "Custom"
name = "Controller"
```

### FMU
```toml
[[tasks]]
type = "Fmu"
name = "PlantModel"
path = "./path/to/library.dylib"
```

### GPIO
```toml
[[tasks]]
type = "Gpio"
name = "Sensors"
pins = [1, 2, 3, 4]
```

### Serial
```toml
[[tasks]]
type = "Serial"
name = "UART_Link"
port = "/dev/ttyUSB0"
baud_rate = 115200
```

### UDP
```toml
[[tasks]]
type = "Udp"
name = "Network"
local_addr = "127.0.0.1:8080"
remote_addr = "127.0.0.1:8081"
```

### Analog
```toml
[[tasks]]
type = "Analog"
name = "DAQ"
channels = [0, 1, 2, 3]
is_input = true
sampling_rate_hz = 10000
```

### ModbusTcp
```toml
[[tasks]]
type = "ModbusTcp"
name = "PLC"
ip_address = "192.168.1.100"
port = 502
```

## How Serde Tag Dispatch Works

The `type` field is used by Serde's tagged enum deserialization:

```rust
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum TaskConfig {
    Custom(CustomConfig),
    Fmu(FmuConfig),
    Gpio(GpioConfig),
    // ...
}
```

Any unrecognized `type` value produces a clear deserialization error.
