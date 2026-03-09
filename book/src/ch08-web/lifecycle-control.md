# Lifecycle Control

The `/control` endpoint manages the simulation's execution state.

## Commands

```bash
# Pause the simulation
curl http://127.0.0.1:3030/control?cmd=pause

# Resume the simulation
curl http://127.0.0.1:3030/control?cmd=resume

# Stop the simulation
curl http://127.0.0.1:3030/control?cmd=stop
```

## State Machine

```
          start
            │
            ▼
  ┌──────────────────┐
  │     Running      │ ◄────── resume
  └──────┬───────────┘
         │ pause
         ▼
  ┌──────────────────┐
  │     Paused       │
  └──────┬───────────┘
         │ stop (or resume → Running → stop)
         ▼
  ┌──────────────────┐
  │     Stopped      │
  └──────────────────┘
```

## Use Cases

- **Debugging** — Pause the simulation to inspect state via `/data` and `/parameters`
- **Parameter changes** — Pause, modify parameters, then resume
- **Graceful shutdown** — Stop the simulation cleanly, ensuring all data is flushed

When the simulation is **paused**:
- The simulation loop continues ticking but skips task execution
- The web server remains responsive
- Data logging is paused

When **stopped**:
- The simulation loop exits
- CSV data is flushed
- The process terminates cleanly

> **Exercise:** Start a simulation with a 30-second duration. Pause it after 5 seconds, wait, then resume. Observe the simulation time in the `/data` endpoint — does it continue from where it paused?
