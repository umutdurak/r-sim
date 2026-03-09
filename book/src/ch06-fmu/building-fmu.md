# Building a Test FMU

Let's build a complete test FMU from scratch. This hands-on exercise demonstrates how external models integrate with r-sim.

## Creating the FMU Crate

r-sim includes a `fmu_test` crate that serves as a minimal FMU:

```rust
// fmu_test/src/lib.rs

#[no_mangle]
pub extern "C" fn do_step(time: f64) -> f64 {
    // Simple model: output increases linearly with time
    time + 1.0
}
```

The `Cargo.toml` must specify `cdylib` to produce a C-compatible dynamic library:

```toml
# fmu_test/Cargo.toml
[package]
name = "fmu_test"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]
```

## Building It

```bash
cargo build
```

The library is produced at `target/debug/deps/libfmu_test.dylib`.

## Using It

```toml
# fmu_config.toml
[[tasks]]
type = "Fmu"
name = "TestModel"
path = "./target/debug/deps/libfmu_test.dylib"
```

```bash
./target/debug/r-sim run -c fmu_config.toml -s 3 -t 500
```

You'll see:

```
Library loaded successfully for task: TestModel
Executing FMU Task TestModel at time 0.5s
  FMU result: 1.5
```

## A More Realistic Example

Let's create a temperature dynamics model:

```rust
// A simple thermal model: dT/dt = (T_ambient - T) / tau + Q / C
// Using Euler integration: T[n+1] = T[n] + dt * dT/dt

static mut TEMPERATURE: f64 = 20.0; // Initial temperature (°C)

#[no_mangle]
pub extern "C" fn do_step(time: f64) -> f64 {
    let dt = 0.5;        // Time step (must match r-sim config)
    let t_ambient = 20.0; // Ambient temperature
    let tau = 10.0;       // Thermal time constant (seconds)
    let q_dot = 50.0;     // Heat input (Watts)
    let c = 100.0;        // Thermal capacity (J/°C)
    
    unsafe {
        let dt_dt = (t_ambient - TEMPERATURE) / tau + q_dot / c;
        TEMPERATURE += dt * dt_dt;
        TEMPERATURE
    }
}
```

This model shows a temperature rising from 20°C toward an equilibrium point, with the rate governed by the thermal time constant.

> **Warning:** Using `static mut` for state in C-exported functions is `unsafe` and not thread-safe. In production, you would use a proper state management approach (e.g., passing a context pointer). This example prioritizes clarity over safety for educational purposes.

> **Exercise:** Modify the thermal model to accept the heat input as a parameter (via the `gain` parameter). Using the web API, change the heat input at runtime and observe how the temperature response changes.
