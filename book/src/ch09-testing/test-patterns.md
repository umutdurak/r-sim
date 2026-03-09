# Test Patterns for Co-Simulation

Testing co-simulation systems requires patterns beyond simple unit testing. Here are strategies applicable to r-sim and real-time simulation in general.

## 1. Configuration Smoke Tests

Verify that every valid configuration loads and runs without errors:

```rust
#[tokio::test]
async fn test_config_loads() {
    let (success, _, _) = run_rsim(&["run", "-c", "my_config.toml", "-s", "1", "-t", "500"]).await;
    assert!(success, "Config should load and run");
}
```

## 2. Dependency Order Verification

Assert that tasks execute in the correct topological order by checking stdout ordering:

```rust
let a_pos = stdout.find("Executing Custom Task A").unwrap();
let b_pos = stdout.find("Executing Custom Task B").unwrap();
assert!(a_pos < b_pos, "A should execute before B");
```

## 3. Error Path Testing (Robustness)

Verify that invalid inputs produce clear errors rather than crashes:

```rust
// Invalid TOML
let (success, _, stderr) = run_rsim(&["run", "-c", "invalid.toml", "-s", "1"]).await;
assert!(!success, "Should fail for invalid config");
assert!(stderr.contains("error"), "Should report parse error");
```

## 4. Web API Testing

Use `reqwest` to verify the web endpoints while the simulation runs:

```rust
let client = reqwest::Client::new();
let resp = client.get("http://127.0.0.1:3030/data").send().await.unwrap();
let body = resp.text().await.unwrap();
assert!(body.contains("current_time_secs"));
```

## 5. Regression Testing with Scenarios

Save known-good configurations as scenarios and re-run them after code changes. Compare CSV output against a reference file.

## The Testing Pyramid for Simulation

```
        ▲
       / \
      / HIL \        Few, expensive, slow
     /  Tests \
    /-----------\
   / Integration  \   Medium effort, subprocess tests
  /    Tests       \
 /-------------------\
/    Unit Tests       \  Many, fast, isolated
/______________________\
```

- **Unit tests** — Test individual task implementations in isolation
- **Integration tests** — Test the full framework with subprocess execution
- **HIL tests** — Test with real hardware (requires lab equipment)
