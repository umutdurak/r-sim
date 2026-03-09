# Distributed Co-Simulation

In large-scale systems, a single machine may not have enough computational power (or the right domain tools) to simulate all subsystems. **Distributed co-simulation** runs simulation models on multiple machines, connected via a network.

## Challenges

1. **Synchronization** — All simulators must advance in lockstep
2. **Communication latency** — Network delays introduce timing uncertainty
3. **Data consistency** — Ensuring all simulators use consistent data at each time step
4. **Clock skew** — Different machines may have slightly different clock rates
5. **Fault tolerance** — What happens when one simulator crashes?

## Standards

Several standards address distributed co-simulation:

| Standard | Organization | Focus |
|----------|-------------|-------|
| **HLA** (High Level Architecture) | IEEE 1516 | Defense, training |
| **DIS** (Distributed Interactive Simulation) | IEEE 1278 | Military simulation |
| **FMI** | Modelica Association | Model exchange |
| **SSP** (System Structure & Parameterization) | Modelica Association | System composition |

## Distributed r-sim (Future)

A distributed version of r-sim would:

1. Run task subsets on different machines
2. Synchronize via a master node
3. Exchange data between nodes via UDP
4. Use the existing dependency graph to determine communication patterns

```
┌────────────┐   UDP    ┌────────────┐   UDP    ┌────────────┐
│  Machine 1 │ ◄──────► │  Machine 2 │ ◄──────► │  Machine 3 │
│  - Engine  │          │  - Trans.  │          │  - Wheels  │
│  - Cooling │          │  - Clutch  │          │  - Brakes  │
└────────────┘          └────────────┘          └────────────┘
                             │
                        ┌────┴────┐
                        │ Master  │
                        │  Node   │
                        └─────────┘
```

> **Exercise (Thought):** How would you partition a 10-task simulation across 3 machines to minimize inter-machine communication? Consider the dependency graph structure.
