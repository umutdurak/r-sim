# UDP Networking

**UDP** (User Datagram Protocol) is a lightweight network protocol commonly used in real-time systems where low latency is more important than guaranteed delivery.

## Configuration

```toml
[[tasks]]
type = "Udp"
name = "DataLink"
local_addr = "127.0.0.1:8080"
remote_addr = "127.0.0.1:8081"
```

## Why UDP for Real-Time?

| Property | TCP | UDP |
|----------|-----|-----|
| Delivery guarantee | Yes (retransmits) | No |
| Ordering guarantee | Yes | No |
| Latency | Higher (handshake + retransmit) | Lower (fire-and-forget) |
| Overhead | 20+ bytes header | 8 bytes header |

Real-time systems prefer UDP because:
- A retransmitted packet arriving late is **worse than useless** — the data is stale
- The simulation will produce a new value on the next time step anyway
- Lower latency means tighter control loops

## Typical Applications

- **Flight simulation** data links (DIS/HLA protocols)
- **Autonomous vehicle** sensor data streaming
- **Distributed simulation** inter-node communication
- **Remote monitoring** data transmission

> **Exercise:** Configure two UDP tasks — one on port 18080 and one on 18081 — representing a sender and receiver. Connect them with a dependency. What would happen in a real system if UDP packets were dropped?
