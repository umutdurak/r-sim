# Real-Time Monitoring

The `/data` and `/graph` endpoints provide live simulation state.

## The `/data` Endpoint

```bash
curl http://127.0.0.1:3030/data
```

Returns:
```json
{
  "current_time_secs": 5.5,
  "task_execution_times_micros": {
    "Controller": 42,
    "PlantModel": 156
  }
}
```

The `task_execution_times_micros` field reports the wall-clock execution time of each task in microseconds. This is invaluable for:
- Detecting overruns (task time > time step)
- Identifying performance bottlenecks
- Benchmarking model complexity

## The `/graph` Endpoint

```bash
curl http://127.0.0.1:3030/graph
```

Returns the task dependency graph:
```json
{
  "tasks": ["Sensor", "Controller", "Actuator"],
  "dependencies": [
    {"from": "Sensor", "to": "Controller"},
    {"from": "Controller", "to": "Actuator"}
  ]
}
```

This can be visualized in any graph rendering tool or a custom web dashboard.

## The `/parameters` Endpoint

```bash
curl http://127.0.0.1:3030/parameters
```

Returns all task parameters:
```json
{
  "Controller": {
    "custom_output": {"Float": 42.0}
  },
  "PlantModel": {
    "gain": {"Float": 1.0},
    "input_var": {"Float": 0.0}
  }
}
```

> **Exercise:** Start a simulation with multiple tasks and poll the `/data` endpoint every second using `watch -n 1 'curl -s http://127.0.0.1:3030/data'`. Observe how execution times vary between time steps.
