# Scenario Management

Scenarios allow you to save, load, and manage simulation configurations for repeatable experiments.

## Saving a Scenario

```bash
./target/debug/r-sim scenario save my_experiment -c experiment_config.toml
```

This copies the configuration to `scenarios/my_experiment.toml`.

## Loading and Running

```bash
./target/debug/r-sim scenario load my_experiment -s 10 -t 100
```

This loads the saved configuration and runs it with the specified duration and time step.

## Listing Scenarios

```bash
./target/debug/r-sim scenario list
```

Output:
```
Available scenarios:
  my_experiment
  baseline_test
  fault_scenario_01
```

## Use Cases

- **Regression testing** — Save a known-good configuration and re-run it after changes
- **Parameter studies** — Save multiple variants of a configuration with different parameters
- **Lab exercises** — Distribute pre-configured scenarios to students
- **Demonstrations** — Prepare scenarios that showcase specific features

> **Exercise:** Create three scenario variants of the same simulation with different `time_multiplier` values (0.5, 1.0, 2.0). Save each as a scenario and compare the execution times.
