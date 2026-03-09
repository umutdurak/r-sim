# Logging Configuration

r-sim can log simulation data to CSV files for post-processing and analysis.

## Configuration

```toml
[logging]
log_file = "simulation_log.csv"
log_interval_millis = 100
logged_outputs = { Controller = ["custom_output"], Plant = ["output_var"] }
```

| Field | Description |
|-------|-------------|
| `log_file` | Output CSV file path |
| `log_interval_millis` | How often to write a row (in ms) |
| `logged_outputs` | Map of task names to output signal names |

## CSV Format

The output CSV contains:
- A `time` column with simulation time
- One column per logged signal, named `task_name.signal_name`

```csv
time,Controller.custom_output,Plant.output_var
0.0,0.0,1.0
0.1,1.0,1.1
0.2,2.0,1.2
```

## Post-Processing

The CSV output can be loaded into any analysis tool:

```python
import pandas as pd
import matplotlib.pyplot as plt

df = pd.read_csv("simulation_log.csv")
df.plot(x="time", y=["Controller.custom_output"])
plt.show()
```

> **Exercise:** Configure logging with `log_interval_millis = 50` and a simulation time step of 100ms. How many rows will the CSV have for a 10-second simulation? What if `log_interval_millis = 200`?
