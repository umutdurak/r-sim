# Topological Sorting

When you define dependencies between tasks, r-sim must determine a valid **execution order** — one where every task runs after all of its predecessors. This is the classic problem of **topological sorting** on a directed acyclic graph (DAG).

## The Algorithm

Given tasks and their dependencies, r-sim uses the [petgraph](https://docs.rs/petgraph/) library to:

1. Build a **directed graph** where nodes are tasks and edges are dependencies
2. Run **topological sort** (Kahn's algorithm) to produce a linear ordering
3. Execute tasks in that order each time step

### Example

Given this configuration:

```toml
[[tasks]]
type = "Custom"
name = "A"

[[tasks]]
type = "Custom"
name = "B"

[[tasks]]
type = "Custom"
name = "C"

[[dependencies]]
from = "A"
to = "B"
type = "direct"

[[dependencies]]
from = "A"
to = "C"
type = "direct"

[[dependencies]]
from = "B"
to = "C"
type = "direct"
```

The dependency graph looks like:

```
A → B → C
A ------↗
```

The topological sort produces: **[A, B, C]** — the only valid ordering.

## Visualizing the Graph

While the simulation is running, query the `/graph` endpoint:

```bash
curl http://127.0.0.1:3030/graph
```

This returns a JSON representation:

```json
{
  "tasks": ["A", "B", "C"],
  "dependencies": [
    {"from": "A", "to": "B"},
    {"from": "A", "to": "C"},
    {"from": "B", "to": "C"}
  ]
}
```

## What Happens Without Dependencies?

If no dependencies are defined, all tasks are treated as independent. The topological sort places them in **arbitrary but deterministic** order (typically the order they appear in the config). Since independent tasks don't exchange data, execution order doesn't affect correctness.

## Cycle Detection

Topological sort is only possible on **acyclic** graphs. If the graph contains a cycle (e.g., A → B → A), topological sort fails!

```
Error: Graph error: Cycle detected. Cannot perform topological sort.
```

This brings us to the next topic: causal loops and how to break them.

> **Exercise:** Create a configuration with 4 tasks (A, B, C, D) where A depends on nothing, B depends on A, C depends on A, and D depends on both B and C. Predict the possible execution orders, then run r-sim to verify.
