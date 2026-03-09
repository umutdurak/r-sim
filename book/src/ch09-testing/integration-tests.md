# Writing Integration Tests

r-sim uses Rust's built-in test framework with `tokio` for async test support. Integration tests run the full `r-sim` binary as a subprocess and verify its behavior.

## Test Structure

```rust
#[tokio::test]
async fn test_basic_simulation() {
    // 1. Create a temporary TOML config
    let config = r#"
[[tasks]]
type = "Custom"
name = "TestTask"
"#;
    tokio::fs::write("/tmp/test_config.toml", config).await.unwrap();

    // 2. Run r-sim as a subprocess
    let output = Command::new("target/debug/r-sim")
        .args(["run", "-c", "/tmp/test_config.toml", "-s", "1", "-t", "500"])
        .output()
        .expect("Failed to run r-sim");

    // 3. Assert on exit code and output
    assert!(output.status.success(), "Should exit successfully");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Executing Custom Task TestTask"));

    // 4. Clean up
    tokio::fs::remove_file("/tmp/test_config.toml").await.ok();
}
```

## Running Tests

```bash
# Run all integration tests (single-threaded due to port 3030)
cargo test --test integration_test -- --test-threads=1
```

The `--test-threads=1` flag is required because each test spawns the web server on port 3030. Running tests in parallel would cause port conflicts.

## Test Categories

r-sim's test suite is organized into categories:

| Category | Tests | What They Verify |
|----------|-------|------------------|
| Core | 7 | Task execution, parallelism, dependencies, causal loops, time sync |
| FMU | 3 | Library loading, parameter tuning, model integration |
| I/O | 8 | GPIO, Serial, UDP, Analog, ModbusTcp, synchronized I/O |
| Config | 6 | TOML loading, lifecycle control, CSV logging, CLI, scenarios |
| Robustness | 2 | Invalid config, missing FMU |

> **Exercise:** Add a new test that verifies a simulation with 3 dependent tasks (A → B → C) produces output lines showing A executing before B, and B before C.
