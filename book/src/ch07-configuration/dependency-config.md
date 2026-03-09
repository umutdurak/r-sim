# Dependency Configuration

Dependencies define the data flow and execution order between tasks.

```toml
[[dependencies]]
from = "Producer"
to = "Consumer"
type = "direct"
data_flow = "output_to_input"
```

## Fields

| Field | Required | Values | Description |
|-------|----------|--------|-------------|
| `from` | Yes | Task name | The upstream task |
| `to` | Yes | Task name | The downstream task |
| `type` | Yes | `"direct"` or `"memory_block"` | Whether data flows instantaneously or with one-step delay |
| `data_flow` | Yes | `"output_to_input"` | Direction of data flow |

## Examples

### Linear Chain
```toml
[[dependencies]]
from = "A"
to = "B"
type = "direct"
data_flow = "output_to_input"

[[dependencies]]
from = "B"
to = "C"
type = "direct"
data_flow = "output_to_input"
```
Execution order: A → B → C

### Feedback Loop with Memory Block
```toml
[[dependencies]]
from = "Controller"
to = "Plant"
type = "direct"
data_flow = "output_to_input"

[[dependencies]]
from = "Plant"
to = "Controller"
type = "memory_block"
data_flow = "output_to_input"
```
Execution order: Controller → Plant (Plant's output feeds back to Controller with one-step delay)

## Common Mistakes

1. **Cyclic direct dependencies** — Use `memory_block` for at least one edge in any cycle
2. **Misspelled task names** — The `from`/`to` values must exactly match a task's `name`
3. **Missing tasks** — All referenced tasks must be defined in `[[tasks]]`
