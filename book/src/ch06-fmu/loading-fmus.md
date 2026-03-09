# Loading External Models

r-sim uses Rust's `libloading` crate to dynamically load FMU libraries at runtime. This allows external models to be compiled separately and loaded without recompiling the framework.

## How FMU Loading Works

1. **Compilation** — The external model is compiled as a C-compatible dynamic library (`.dylib` on macOS, `.so` on Linux, `.dll` on Windows)
2. **Configuration** — The library path is specified in the TOML config
3. **Loading** — r-sim loads the library and finds the `do_step` symbol
4. **Execution** — The function is called with the current time each time step

```toml
[[tasks]]
type = "Fmu"
name = "PlantModel"
path = "./target/debug/deps/libfmu_test.dylib"
```

## The `do_step` Function

At minimum, an FMU must export a C function:

```c
#[no_mangle]
pub extern "C" fn do_step(time: f64) -> f64 {
    // Compute and return the model's output
    time + 1.0
}
```

The `#[no_mangle]` attribute prevents Rust from changing the function name, and `extern "C"` makes it compatible with the C ABI. The function receives the current simulation time and returns a computed value.

## FmuTask Internals

```rust
pub struct FmuTask {
    name: String,
    inputs: HashMap<String, f64>,
    outputs: HashMap<String, f64>,
    _library: Library,           // Keeps the library loaded
    do_step_fn: Symbol<'static, unsafe extern "C" fn(f64) -> f64>,
    parameters: HashMap<String, Parameter>,
}
```

On each time step:

```rust
async fn execute(&mut self, current_time: Duration) -> Result<(), FrameworkError> {
    let time_secs = current_time.as_secs_f64();
    let result = unsafe { (self.do_step_fn)(time_secs) };
    self.outputs.insert("output_var".to_string(), result);
    let input_val = self.inputs.get("input_var").copied().unwrap_or(0.0);
    let gain = /* read from parameters */;
    self.outputs.insert("output_var".to_string(), result * gain + input_val);
    Ok(())
}
```

## Error Handling

If the library path is invalid or the `do_step` symbol is not found, r-sim reports a clear error:

```
Libloading error: dlopen(...): No such file or directory
```

This is verified by the `tc_rob_002_missing_fmu` integration test.

> **Exercise:** Write a simple `do_step` function in Rust that simulates a first-order system: `y[n] = 0.9 * y[n-1] + 0.1 * input`. What challenges do you face when managing state in a C-exported function?
