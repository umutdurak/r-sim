# Real-Time vs. Non-Real-Time Simulation

Understanding the distinction between real-time and non-real-time simulation is fundamental to choosing the right approach for your problem.

## Comparison

| Property | Non-Real-Time | Real-Time |
|----------|--------------|-----------|
| **Time constraint** | None — runs as fast as possible | Must keep pace with wall clock |
| **Step size** | Often variable, adaptive | Fixed |
| **Primary concern** | Accuracy | Timeliness |
| **Failure mode** | Inaccurate results | Missed deadlines (overrun) |
| **Typical use** | Design exploration, optimization | Testing, training, deployment |
| **Hardware coupling** | Usually none | Often connected to real hardware |

## The Real-Time Spectrum

In practice, real-time systems exist on a spectrum:

### Hard Real-Time

Missing a deadline is a **system failure**. Examples:
- Airbag deployment controllers (must fire within 10ms)
- Anti-lock braking systems (must compute every 5ms)
- Nuclear reactor control (must respond within tight bounds)

### Firm Real-Time

Missing a deadline degrades quality but doesn't cause failure:
- Video streaming (a dropped frame causes a glitch)
- Audio processing (a late sample causes a click)

### Soft Real-Time

Missing a deadline is undesirable but tolerable:
- Web server response times
- Interactive simulations and games
- Monitoring dashboards

r-sim operates in the **soft to firm real-time** range. It uses Tokio's async runtime rather than a hard real-time operating system (RTOS), making it suitable for:

- Software-in-the-loop testing
- Rapid prototyping of control systems
- Educational demonstrations
- System integration testing

For hard real-time applications, you would typically deploy on an RTOS like VxWorks, QNX, or Linux with the PREEMPT_RT patch, and use r-sim's architecture as a design template.

## Model-in-the-Loop to Hardware-in-the-Loop

The simulation fidelity spectrum follows a well-known progression:

```
MIL → SIL → PIL → HIL → Deployment
```

| Stage | Description | Time Constraint |
|-------|-------------|-----------------|
| **MIL** (Model-in-the-Loop) | All models run in a simulation environment | Non-real-time |
| **SIL** (Software-in-the-Loop) | Production code runs against simulated plant | Soft real-time |
| **PIL** (Processor-in-the-Loop) | Code runs on target processor, simulated plant | Firm real-time |
| **HIL** (Hardware-in-the-Loop) | Real controller hardware, simulated plant | Hard real-time |
| **Deployment** | Real controller, real plant | Hard real-time |

r-sim is well-suited for MIL, SIL, and early-stage HIL testing. We'll explore these methodologies in detail in [Chapter 10](../ch10-advanced.md).
