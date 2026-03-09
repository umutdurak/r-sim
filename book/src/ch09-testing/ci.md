# Continuous Integration

Automating test execution in CI ensures that changes don't break existing functionality.

## GitHub Actions Example

```yaml
# .github/workflows/test.yml
name: r-sim Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release
      - run: cargo test --test integration_test -- --test-threads=1
```

## Best Practices

1. **Run tests on every PR** — Catch regressions early
2. **Use `--test-threads=1`** — Avoid port conflicts
3. **Build before testing** — Ensure the binary is up to date
4. **Cache cargo dependencies** — Speed up CI runs
5. **Test on multiple platforms** — macOS, Linux, Windows
