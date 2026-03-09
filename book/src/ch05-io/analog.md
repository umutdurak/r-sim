# Analog I/O and Data Acquisition

**Analog I/O** interfaces convert between the continuous-valued signals of the physical world and the discrete values used by digital computation. Analog-to-Digital Converters (ADCs) sample physical signals, while Digital-to-Analog Converters (DACs) produce analog outputs.

## Configuration

```toml
[[tasks]]
type = "Analog"
name = "TemperatureSensors"
channels = [0, 1, 2, 3]
is_input = true
sampling_rate_hz = 10000
```

| Parameter | Description |
|-----------|-------------|
| `channels` | List of ADC/DAC channel numbers |
| `is_input` | `true` for ADC (reading), `false` for DAC (writing) |
| `sampling_rate_hz` | Optional: sampling rate for high-speed acquisition |

## High-Speed Data Acquisition

In some applications, analog signals must be sampled at very high rates — much faster than the simulation time step:

| Application | Required Sample Rate |
|-------------|---------------------|
| Vibration analysis | 10–100 kHz |
| Audio processing | 44.1–192 kHz |
| Power electronics | 1–10 MHz |
| Industrial temperature | 10–100 Hz |

The `sampling_rate_hz` parameter configures the DAQ task for high-speed acquisition. While the simulation loop runs at the time step rate, the analog task can buffer multiple samples between time steps.

## The Nyquist Criterion

A fundamental theorem in signal processing: to faithfully represent a signal, you must sample at **at least twice** its highest frequency component:

```
f_sample ≥ 2 × f_max
```

If your system has dynamics up to 1 kHz, you need at least a 2 kHz sampling rate. In practice, sampling at 5–10× the maximum frequency is preferred.

> **Exercise:** A vibration sensor produces signals up to 5 kHz. What minimum `sampling_rate_hz` would you configure? What would you see in the data if you undersampled?
