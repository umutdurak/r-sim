# Parameter Tuning at Runtime

The `/parameters/set` endpoint allows you to modify task parameters while the simulation is running — without stopping, reconfiguring, and restarting.

## Setting a Parameter

```bash
curl -X POST http://127.0.0.1:3030/parameters/set \
  -H "Content-Type: application/json" \
  -d '{"task_name": "PlantModel", "param_name": "gain", "param_value": {"Float": 2.5}}'
```

Response: `"Parameter set successfully."`

## Parameter Types

| Type | JSON Format | Example |
|------|------------|---------|
| Float | `{"Float": 3.14}` | Gains, coefficients |
| Integer | `{"Integer": 42}` | Counts, indices |
| Boolean | `{"Boolean": true}` | Enable/disable flags |
| String | `{"String": "high"}` | Mode selectors |

## Practical Use Cases

1. **Gain tuning** — Adjust PID gains in a controller until the response is satisfactory
2. **Fault injection** — Change a parameter to simulate sensor failure (e.g., set offset to a large value)
3. **What-if analysis** — Change environmental parameters mid-simulation
4. **Student exercises** — Let students explore parameter effects interactively

## Workflow

```
1. Start simulation
2. Query /parameters to see current values
3. Adjust a parameter via /parameters/set
4. Observe the effect via /data
5. Iterate until desired behavior is achieved
```

> **Exercise:** Start a simulation with an FMU task. Use `curl` to change the `gain` parameter from 1.0 to 3.0. Query `/parameters` before and after to verify the change took effect.
