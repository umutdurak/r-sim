# Serial Communication

**Serial communication** (UART/RS-232) is one of the most common interfaces for embedded systems. Many sensors, actuators, and microcontrollers communicate over serial ports.

## Configuration

```toml
[[tasks]]
type = "Serial"
name = "SensorModule"
port = "/dev/ttyUSB0"
baud_rate = 115200
```

| Parameter | Description | Common Values |
|-----------|-------------|---------------|
| `port` | Serial port device path | `/dev/ttyUSB0`, `COM3` |
| `baud_rate` | Communication speed in bits/sec | 9600, 115200, 921600 |

## How It Works

The serial task simulates a full-duplex serial connection:
- **`read_inputs()`** reads incoming bytes from the port
- **`execute()`** processes the received data
- **`write_outputs()`** transmits computed responses

## Typical Applications

- **GPS receivers** — NMEA sentences at 9600 baud
- **IMU sensors** — Binary data at 115200 baud
- **Modems** — AT commands at various rates
- **Arduino/microcontroller** — Custom protocols

> **Exercise:** Configure a serial task at 9600 baud and another at 115200 baud. Why might the choice of baud rate affect your simulation's time step selection?
