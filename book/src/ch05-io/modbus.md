# Modbus TCP for Industrial Systems

**Modbus** is one of the oldest and most widely used industrial communication protocols. Originally developed by Modicon in 1979, it remains the de facto standard for connecting industrial electronic devices.

## What Is Modbus?

Modbus defines a simple **master/slave** (client/server) protocol for reading and writing registers. Modbus TCP wraps this protocol in TCP/IP packets, allowing it to run over standard Ethernet networks.

### Register Types

| Register Type | Access | Typical Use |
|--------------|--------|-------------|
| Coils | Read/Write | Digital outputs (relays, switches) |
| Discrete Inputs | Read-only | Digital inputs (sensors) |
| Holding Registers | Read/Write | 16-bit analog values, setpoints |
| Input Registers | Read-only | 16-bit analog inputs (measurements) |

## Configuration

```toml
[[tasks]]
type = "ModbusTcp"
name = "PLC_Interface"
ip_address = "192.168.1.100"
port = 502
```

Port 502 is the standard Modbus TCP port.

## Typical Industrial Setup

In a HIL testing scenario:

```
┌─────────────┐    Modbus TCP    ┌───────────┐
│  r-sim      │ ◄──────────────► │   PLC     │
│  (Plant     │   Holding Regs   │ (Control  │
│   Model)    │   Input Regs     │  System)  │
└─────────────┘                  └───────────┘
```

- r-sim simulates the plant and writes sensor values to **Input Registers**
- The PLC reads those registers, computes control logic, and writes to **Holding Registers**
- r-sim reads the Holding Registers as actuator commands

> **Exercise:** A PLC controls a motor via holding register address 40001 (speed setpoint). A tachometer provides speed feedback via input register 30001. Sketch how you would configure r-sim tasks and dependencies to simulate this system.
