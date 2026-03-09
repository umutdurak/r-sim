# Fault Injection Testing

**Fault injection** is the deliberate introduction of errors into a system to test its robustness and failure handling. In simulation-based testing, faults can be injected into models, I/O signals, or communication channels.

## Types of Faults

| Fault Type | Description | Example |
|-----------|-------------|---------|
| Sensor failure | A sensor produces incorrect or stuck values | Temperature sensor reads 0°C always |
| Actuator failure | An actuator doesn't respond to commands | Motor stuck at current speed |
| Communication loss | Data link interrupted | UDP packets dropped |
| Model degradation | A component operates outside normal parameters | Increased friction coefficient |
| Timing failure | A task takes longer than expected | Overrun simulation |

## Implementing Fault Injection

In r-sim, faults can be injected through runtime parameter tuning:

```bash
# Simulate a sensor stuck at 0.0
curl -X POST http://127.0.0.1:3030/parameters/set \
  -d '{"task_name": "Sensor", "param_name": "gain", "param_value": {"Float": 0.0}}'
```

A more comprehensive fault injection system would include:
- **Scheduled faults** — Automatically inject at specific simulation times
- **Random faults** — Probabilistic fault occurrence
- **Compound faults** — Multiple simultaneous failures
- **Fault recovery** — Testing system's ability to recover

## Verification

After injecting a fault:
1. Monitor the system's response via the web API
2. Verify that safety mechanisms activate
3. Check that the system degrades gracefully
4. Ensure fault detection algorithms identify the failure

> **Exercise (Design):** Design a fault injection mechanism for r-sim. How would you specify faults in the TOML configuration? What kind of fault scheduling would be most useful?
