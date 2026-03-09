use std::process::{Command, Stdio};
use std::path::PathBuf;
use std::io::{BufReader, Read};
use tokio::time::{timeout, Duration};

/// Helper: run r-sim with given args and return (exit_success, stdout, stderr)
async fn run_rsim(args: &[&str]) -> (bool, String, String) {
    let mut child = Command::new("target/debug/r-sim")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn r-sim process");

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let stdout_task = tokio::task::spawn_blocking(move || {
        let mut reader = BufReader::new(stdout);
        let mut buf = String::new();
        reader.read_to_string(&mut buf).unwrap();
        buf
    });

    let stderr_task = tokio::task::spawn_blocking(move || {
        let mut reader = BufReader::new(stderr);
        let mut buf = String::new();
        reader.read_to_string(&mut buf).unwrap();
        buf
    });

    let child_id = child.id();
    let wait_result = timeout(
        Duration::from_secs(15),
        tokio::task::spawn_blocking(move || child.wait()),
    )
    .await;

    match wait_result {
        Ok(Ok(status)) => {
            let out = stdout_task.await.unwrap();
            let err = stderr_task.await.unwrap();
            let exit_status = status.expect("Failed to get exit status");
            (exit_status.success(), out, err)
        }
        _ => {
            // Kill on timeout
            let _ = Command::new("kill").arg(child_id.to_string()).output();
            let out = stdout_task.await.unwrap_or_default();
            let err = stderr_task.await.unwrap_or_default();
            (false, out, err)
        }
    }
}

// ============================================================
// TC-CORE-001: Basic simulation execution
// Verifies: REQ-SIMULATION-EXECUTION, REQ-PERIODIC-TASKS, REQ-TIME-STEP-CONTROL
// ============================================================
#[tokio::test]
async fn tc_core_001_basic_simulation_execution() {
    let config = r#"
[[tasks]]
type = "Custom"
name = "TestTask"
"#;
    let config_path = PathBuf::from("/tmp/tc_core_001.toml");
    tokio::fs::write(&config_path, config).await.unwrap();

    let (success, stdout, _stderr) = run_rsim(&[
        "run", "-c", config_path.to_str().unwrap(), "-s", "2", "-t", "500",
    ])
    .await;

    assert!(success, "Simulation should exit successfully");
    assert!(
        stdout.contains("Simulation finished"),
        "Should complete simulation. Got: {}",
        &stdout[stdout.len().saturating_sub(200)..]
    );
    assert!(
        stdout.contains("Creating task: Custom"),
        "Should create CustomTask from config"
    );
    assert!(
        stdout.contains("Executing Custom Task TestTask"),
        "Should execute the task during simulation"
    );

    tokio::fs::remove_file(&config_path).await.ok();
}

// ============================================================
// TC-CORE-002: Parallel execution of independent tasks
// Verifies: REQ-PARALLEL-EXECUTION
// ============================================================
#[tokio::test]
async fn tc_core_002_parallel_execution() {
    let config = r#"
[[tasks]]
type = "Custom"
name = "IndependentA"

[[tasks]]
type = "Custom"
name = "IndependentB"
"#;
    let config_path = PathBuf::from("/tmp/tc_core_002.toml");
    tokio::fs::write(&config_path, config).await.unwrap();

    let (success, stdout, _) = run_rsim(&[
        "run", "-c", config_path.to_str().unwrap(), "-s", "1", "-t", "500",
    ])
    .await;

    assert!(success, "Simulation should exit successfully");
    assert!(stdout.contains("Creating task: Custom(CustomConfig { name: \"IndependentA\" })"));
    assert!(stdout.contains("Creating task: Custom(CustomConfig { name: \"IndependentB\" })"));
    assert!(stdout.contains("Execution order determined"));

    tokio::fs::remove_file(&config_path).await.ok();
}

// ============================================================
// TC-CORE-003: Data flow and execution order
// Verifies: REQ-DATA-FLOW, REQ-EXECUTION-ORDER
// ============================================================
#[tokio::test]
async fn tc_core_003_data_flow_execution_order() {
    let config = r#"
[[tasks]]
type = "Custom"
name = "Producer"

[[tasks]]
type = "Custom"
name = "Consumer"

[[dependencies]]
from = "Producer"
to = "Consumer"
type = "direct"
data_flow = "output_to_input"
"#;
    let config_path = PathBuf::from("/tmp/tc_core_003.toml");
    tokio::fs::write(&config_path, config).await.unwrap();

    let (success, stdout, _) = run_rsim(&[
        "run", "-c", config_path.to_str().unwrap(), "-s", "1", "-t", "500",
    ])
    .await;

    assert!(success, "Simulation should succeed with dependencies");
    assert!(
        stdout.contains("Adding dependency"),
        "Should parse dependency config"
    );
    assert!(
        stdout.contains("Execution order determined"),
        "Topological sort should succeed"
    );

    // Verify execution order: Producer must appear before Consumer in the output
    let producer_pos = stdout.find("Executing Custom Task Producer");
    let consumer_pos = stdout.find("Executing Custom Task Consumer");
    assert!(producer_pos.is_some(), "Producer should execute");
    assert!(consumer_pos.is_some(), "Consumer should execute");
    assert!(
        producer_pos.unwrap() < consumer_pos.unwrap(),
        "Producer must execute before Consumer (topological order)"
    );

    tokio::fs::remove_file(&config_path).await.ok();
}

// ============================================================
// TC-CORE-004: Causal loop detection + memory blocks
// Verifies: REQ-CAUSAL-LOOP-DETECTION, REQ-MEMORY-BLOCKS
// ============================================================
#[tokio::test]
async fn tc_core_004_causal_loop_memory_block() {
    let config = r#"
[[tasks]]
type = "Custom"
name = "TaskA"

[[tasks]]
type = "Custom"
name = "TaskB"

[[dependencies]]
from = "TaskA"
to = "TaskB"
type = "direct"
data_flow = "output_to_input"

[[dependencies]]
from = "TaskB"
to = "TaskA"
type = "memory_block"
data_flow = "feedback"
"#;
    let config_path = PathBuf::from("/tmp/tc_core_004.toml");
    tokio::fs::write(&config_path, config).await.unwrap();

    let (success, stdout, _) = run_rsim(&[
        "run", "-c", config_path.to_str().unwrap(), "-s", "1", "-t", "500",
    ])
    .await;

    assert!(success, "Memory block should resolve the causal cycle");
    assert!(
        stdout.contains("Execution order determined"),
        "Topological sort should succeed with memory block breaking cycle"
    );
    assert!(
        !stdout.contains("Causal loop detected"),
        "Should NOT report a causal loop error"
    );

    tokio::fs::remove_file(&config_path).await.ok();
}

// ============================================================
// TC-CORE-005: Time synchronization
// Verifies: REQ-TIME-SYNCHRONIZATION
// ============================================================
#[tokio::test]
async fn tc_core_005_time_synchronization() {
    let config = r#"
time_multiplier = 0.5

[[tasks]]
type = "Custom"
name = "SlowTask"
"#;
    let config_path = PathBuf::from("/tmp/tc_core_005.toml");
    tokio::fs::write(&config_path, config).await.unwrap();

    let (success, stdout, _) = run_rsim(&[
        "run", "-c", config_path.to_str().unwrap(), "-s", "1", "-t", "500",
    ])
    .await;

    assert!(success, "Simulation should succeed with time multiplier");
    assert!(
        stdout.contains("Applying time multiplier: 0.5"),
        "Time multiplier should be applied from config"
    );

    tokio::fs::remove_file(&config_path).await.ok();
}

// ============================================================
// TC-CORE-008: Custom component integration via API
// Verifies: REQ-CUSTOM-COMPONENT-INTEGRATION, REQ-IO-EXTENSIBILITY
// ============================================================
#[tokio::test]
async fn tc_core_008_custom_component_integration() {
    let config = r#"
[[tasks]]
type = "Custom"
name = "UserDefinedTask"
"#;
    let config_path = PathBuf::from("/tmp/tc_core_008.toml");
    tokio::fs::write(&config_path, config).await.unwrap();

    let (success, stdout, _) = run_rsim(&[
        "run", "-c", config_path.to_str().unwrap(), "-s", "1", "-t", "500",
    ])
    .await;

    assert!(success);
    assert!(
        stdout.contains("Creating task: Custom(CustomConfig { name: \"UserDefinedTask\" })"),
        "TaskFactory should create custom task from config"
    );
    assert!(
        stdout.contains("Executing Custom Task UserDefinedTask"),
        "Custom task should execute in the simulation loop"
    );

    tokio::fs::remove_file(&config_path).await.ok();
}

// ============================================================
// TC-FMU-001: FMU loading and execution
// Verifies: REQ-FMU-LOADING, REQ-FMU-EXECUTION
// ============================================================
#[tokio::test]
async fn tc_fmu_001_fmu_loading_and_execution() {
    let fmu_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/debug/deps/libfmu_test.dylib");

    if !fmu_path.exists() {
        eprintln!("SKIP: FMU library not found at {:?}", fmu_path);
        return;
    }

    let config = format!(
        r#"
[[tasks]]
type = "Fmu"
name = "TestFMU"
path = "{}"
"#,
        fmu_path.to_str().unwrap()
    );
    let config_path = PathBuf::from("/tmp/tc_fmu_001.toml");
    tokio::fs::write(&config_path, &config).await.unwrap();

    let (success, stdout, stderr) = run_rsim(&[
        "run", "-c", config_path.to_str().unwrap(), "-s", "1", "-t", "500",
    ])
    .await;

    assert!(
        success,
        "FMU simulation should succeed. stderr: {}",
        stderr
    );
    assert!(
        stdout.contains("Library loaded successfully"),
        "FMU library should load"
    );
    assert!(
        stdout.contains("Executing FMU Task TestFMU"),
        "FMU task should execute. stdout tail: {}",
        &stdout[stdout.len().saturating_sub(300)..]
    );

    tokio::fs::remove_file(&config_path).await.ok();
}

// ============================================================
// TC-IO-001: GPIO task
// Verifies: REQ-IO-GPIO
// ============================================================
#[tokio::test]
async fn tc_io_001_gpio_task() {
    let config = r#"
[[tasks]]
type = "Gpio"
name = "GPIO_Test"
pins = [1, 2, 3]
"#;
    let config_path = PathBuf::from("/tmp/tc_io_001.toml");
    tokio::fs::write(&config_path, config).await.unwrap();

    let (success, stdout, _) = run_rsim(&[
        "run", "-c", config_path.to_str().unwrap(), "-s", "1", "-t", "500",
    ])
    .await;

    assert!(success, "GPIO simulation should succeed");
    assert!(stdout.contains("GPIO Task GPIO_Test initialized for pins: [1, 2, 3]"));
    assert!(stdout.contains("GPIO Task GPIO_Test reading inputs"));
    assert!(stdout.contains("GPIO Task GPIO_Test writing outputs"));

    tokio::fs::remove_file(&config_path).await.ok();
}

// ============================================================
// TC-IO-003: UDP task
// Verifies: REQ-IO-UDP
// ============================================================
#[tokio::test]
async fn tc_io_003_udp_task() {
    let config = r#"
[[tasks]]
type = "Udp"
name = "UDP_Test"
local_addr = "127.0.0.1:18080"
remote_addr = "127.0.0.1:18081"
"#;
    let config_path = PathBuf::from("/tmp/tc_io_003.toml");
    tokio::fs::write(&config_path, config).await.unwrap();

    let (success, stdout, _) = run_rsim(&[
        "run", "-c", config_path.to_str().unwrap(), "-s", "1", "-t", "500",
    ])
    .await;

    assert!(success, "UDP simulation should succeed");
    assert!(stdout.contains("UDP Task UDP_Test initialized and bound to 127.0.0.1:18080"));

    tokio::fs::remove_file(&config_path).await.ok();
}

// ============================================================
// TC-IO-004: Analog task
// Verifies: REQ-IO-ANALOG
// ============================================================
#[tokio::test]
async fn tc_io_004_analog_task() {
    let config = r#"
[[tasks]]
type = "Analog"
name = "Analog_Test"
channels = [0, 1]
is_input = true
"#;
    let config_path = PathBuf::from("/tmp/tc_io_004.toml");
    tokio::fs::write(&config_path, config).await.unwrap();

    let (success, stdout, _) = run_rsim(&[
        "run", "-c", config_path.to_str().unwrap(), "-s", "1", "-t", "500",
    ])
    .await;

    assert!(success, "Analog simulation should succeed");
    assert!(stdout.contains("Creating task: Analog"));

    tokio::fs::remove_file(&config_path).await.ok();
}

// ============================================================
// TC-IO-006: Modbus TCP task
// Verifies: REQ-IO-MODBUS
// ============================================================
#[tokio::test]
async fn tc_io_006_modbus_tcp_task() {
    let config = r#"
[[tasks]]
type = "ModbusTcp"
name = "Modbus_Test"
ip_address = "127.0.0.1"
port = 1502
"#;
    let config_path = PathBuf::from("/tmp/tc_io_006.toml");
    tokio::fs::write(&config_path, config).await.unwrap();

    let (success, stdout, _) = run_rsim(&[
        "run", "-c", config_path.to_str().unwrap(), "-s", "1", "-t", "500",
    ])
    .await;

    assert!(success, "ModbusTcp simulation should succeed");
    assert!(stdout.contains("Creating task: ModbusTcp"));

    tokio::fs::remove_file(&config_path).await.ok();
}

// ============================================================
// TC-CONF-001: TOML config loading
// Verifies: REQ-TOML-CONFIG
// ============================================================
#[tokio::test]
async fn tc_conf_001_toml_config_loading() {
    let config = r#"
[[tasks]]
type = "Custom"
name = "ConfigTest"
time_multiplier = 1.0

[logging]
log_file = "/tmp/tc_conf_001_log.csv"
log_interval_millis = 100
logged_outputs = { ConfigTest = ["custom_output"] }
"#;
    let config_path = PathBuf::from("/tmp/tc_conf_001.toml");
    tokio::fs::write(&config_path, config).await.unwrap();

    let (success, stdout, _) = run_rsim(&[
        "run", "-c", config_path.to_str().unwrap(), "-s", "1", "-t", "500",
    ])
    .await;

    assert!(success, "Config loading should succeed");
    assert!(stdout.contains("Reading config from"));
    assert!(stdout.contains("Creating task: Custom"));
    assert!(stdout.contains("CSV headers written"));

    tokio::fs::remove_file(&config_path).await.ok();
    tokio::fs::remove_file("/tmp/tc_conf_001_log.csv").await.ok();
}

// ============================================================
// TC-CONF-003: Data logging to CSV
// Verifies: REQ-DATA-LOGGING
// ============================================================
#[tokio::test]
async fn tc_conf_003_data_logging_to_csv() {
    let log_path = "/tmp/tc_conf_003_simulation_log.csv";
    let config = format!(
        r#"
[[tasks]]
type = "Custom"
name = "LoggedTask"

[logging]
log_file = "{}"
log_interval_millis = 100
logged_outputs = {{ LoggedTask = ["custom_output"] }}
"#,
        log_path
    );
    let config_path = PathBuf::from("/tmp/tc_conf_003.toml");
    tokio::fs::write(&config_path, &config).await.unwrap();

    let (success, _, _) = run_rsim(&[
        "run", "-c", config_path.to_str().unwrap(), "-s", "1", "-t", "500",
    ])
    .await;

    assert!(success, "Simulation should succeed");

    let log_file = PathBuf::from(log_path);
    assert!(log_file.exists(), "CSV log file should be created");

    let content = tokio::fs::read_to_string(&log_file).await.unwrap();
    let lines: Vec<&str> = content.lines().collect();
    assert!(lines.len() >= 2, "CSV should have header + data rows. Got: {}", lines.len());
    assert!(
        lines[0].contains("Time"),
        "First line should be CSV header with 'Time'"
    );

    tokio::fs::remove_file(&config_path).await.ok();
    tokio::fs::remove_file(log_path).await.ok();
}

// ============================================================
// TC-CONF-005: CLI help output
// Verifies: REQ-CLI-INTERFACE
// ============================================================
#[tokio::test]
async fn tc_conf_005_cli_help() {
    let (success, stdout, _) = run_rsim(&["--help"]).await;
    assert!(success, "r-sim --help should succeed");
    assert!(stdout.contains("Usage") || stdout.contains("usage") || stdout.contains("USAGE"));
    assert!(stdout.contains("run"));
    assert!(stdout.contains("control"));
    assert!(stdout.contains("scenario"));
}

// ============================================================
// TC-ROB-001: Invalid config error handling
// Verifies: REQ-ERROR-HANDLING
// ============================================================
#[tokio::test]
async fn tc_rob_001_invalid_config() {
    let config = r#"
this is not valid toml at all {{{
"#;
    let config_path = PathBuf::from("/tmp/tc_rob_001.toml");
    tokio::fs::write(&config_path, config).await.unwrap();

    let (success, _stdout, stderr) = run_rsim(&[
        "run", "-c", config_path.to_str().unwrap(), "-s", "1", "-t", "500",
    ])
    .await;

    assert!(!success, "Invalid config should cause non-zero exit");
    let combined = format!("{}{}", _stdout, stderr);
    assert!(
        combined.contains("error") || combined.contains("Error") || combined.contains("TOML"),
        "Should report a TOML parse error. Got: {}",
        combined
    );

    tokio::fs::remove_file(&config_path).await.ok();
}

// ============================================================
// TC-ROB-002: Non-existent FMU error handling
// Verifies: REQ-ERROR-HANDLING, REQ-FMU-LOADING
// ============================================================
#[tokio::test]
async fn tc_rob_002_missing_fmu() {
    let config = r#"
[[tasks]]
type = "Fmu"
name = "GhostFMU"
path = "/nonexistent/path/to/phantom.dylib"
"#;
    let config_path = PathBuf::from("/tmp/tc_rob_002.toml");
    tokio::fs::write(&config_path, config).await.unwrap();

    let (success, stdout, stderr) = run_rsim(&[
        "run", "-c", config_path.to_str().unwrap(), "-s", "1", "-t", "500",
    ])
    .await;

    assert!(!success, "Missing FMU should cause failure");
    let combined = format!("{}{}", stdout, stderr);
    assert!(
        combined.contains("error") || combined.contains("Error") || combined.contains("not found") || combined.contains("No such file"),
        "Should report a library loading error"
    );

    tokio::fs::remove_file(&config_path).await.ok();
}

// ============================================================
// TC-CONF-006: Scenario save / load / list
// Verifies: REQ-SCENARIO-MANAGEMENT
// ============================================================
#[tokio::test]
async fn tc_conf_006_scenario_management() {
    // Save a scenario
    let config = r#"
[[tasks]]
type = "Custom"
name = "ScenarioTask"
"#;
    let config_path = PathBuf::from("/tmp/tc_conf_006.toml");
    tokio::fs::write(&config_path, config).await.unwrap();

    let (save_ok, save_out, _) = run_rsim(&[
        "scenario", "save", "test_scenario_006", "-c", config_path.to_str().unwrap(),
    ])
    .await;
    assert!(save_ok, "Scenario save should succeed");
    assert!(
        save_out.contains("saved") || save_out.contains("Scenario"),
        "Should confirm scenario was saved"
    );

    // List scenarios
    let (list_ok, list_out, _) = run_rsim(&["scenario", "list"]).await;
    assert!(list_ok, "Scenario list should succeed");
    assert!(
        list_out.contains("test_scenario_006"),
        "Saved scenario should appear in list"
    );

    // Clean up
    tokio::fs::remove_file(&config_path).await.ok();
    tokio::fs::remove_file("scenarios/test_scenario_006.toml").await.ok();
}

// ============================================================
// Helper: start r-sim in background, returning (Child, child_id)
// ============================================================
fn start_rsim_background(config_path: &str, duration_secs: &str) -> (std::process::Child, u32) {
    let child = Command::new("target/debug/r-sim")
        .args(["run", "-c", config_path, "-s", duration_secs, "-t", "500"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn r-sim");
    let id = child.id();
    (child, id)
}

fn kill_child(child_id: u32) {
    let _ = Command::new("kill").arg(child_id.to_string()).output();
}

// ============================================================
// TC-CORE-006: Performance metrics reporting
// Verifies: REQ-PERFORMANCE-METRICS
// ============================================================
#[tokio::test]
async fn tc_core_006_performance_metrics_via_web() {
    let config = r#"
[[tasks]]
type = "Custom"
name = "MetricsTask"
"#;
    let config_path = "/tmp/tc_core_006.toml";
    tokio::fs::write(config_path, config).await.unwrap();

    let (mut child, child_id) = start_rsim_background(config_path, "30");
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Query the /data endpoint for execution time metrics
    let client = reqwest::Client::new();
    let resp = client.get("http://127.0.0.1:3030/data")
        .timeout(Duration::from_secs(3))
        .send().await;

    kill_child(child_id);
    let _ = child.wait();

    match resp {
        Ok(r) => {
            let body = r.text().await.unwrap_or_default();
            assert!(
                body.contains("task_execution_times_micros"),
                "Web /data endpoint should include task_execution_times_micros. Got: {}",
                &body[..body.len().min(300)]
            );
            assert!(
                body.contains("current_time_secs"),
                "/data should include current_time_secs"
            );
        }
        Err(e) => {
            panic!("Failed to reach /data endpoint: {}", e);
        }
    }

    tokio::fs::remove_file(config_path).await.ok();
}

// ============================================================
// TC-CONF-002: Lifecycle control (pause/resume/stop)
// Verifies: REQ-LIFECYCLE-CONTROL
// ============================================================
#[tokio::test]
async fn tc_conf_002_lifecycle_control() {
    let config = r#"
[[tasks]]
type = "Custom"
name = "LifecycleTask"
"#;
    let config_path = "/tmp/tc_conf_002.toml";
    tokio::fs::write(config_path, config).await.unwrap();

    let (mut child, child_id) = start_rsim_background(config_path, "30");
    tokio::time::sleep(Duration::from_secs(3)).await;

    let client = reqwest::Client::new();

    // Pause
    let pause_resp = client.get("http://127.0.0.1:3030/control?cmd=pause")
        .timeout(Duration::from_secs(3))
        .send().await;
    assert!(pause_resp.is_ok(), "Pause command should succeed");
    let pause_body = pause_resp.unwrap().text().await.unwrap_or_default();
    assert!(
        pause_body.contains("paused") || pause_body.contains("Paused"),
        "Should confirm pause. Got: {}",
        pause_body
    );

    tokio::time::sleep(Duration::from_millis(500)).await;

    // Resume
    let resume_resp = client.get("http://127.0.0.1:3030/control?cmd=resume")
        .timeout(Duration::from_secs(3))
        .send().await;
    assert!(resume_resp.is_ok(), "Resume command should succeed");
    let resume_body = resume_resp.unwrap().text().await.unwrap_or_default();
    assert!(
        resume_body.contains("resumed") || resume_body.contains("Resumed"),
        "Should confirm resume. Got: {}",
        resume_body
    );

    tokio::time::sleep(Duration::from_millis(500)).await;

    // Stop
    let stop_resp = client.get("http://127.0.0.1:3030/control?cmd=stop")
        .timeout(Duration::from_secs(3))
        .send().await;
    assert!(stop_resp.is_ok(), "Stop command should succeed");
    let stop_body = stop_resp.unwrap().text().await.unwrap_or_default();
    assert!(
        stop_body.contains("stopped") || stop_body.contains("Stopped"),
        "Should confirm stop. Got: {}",
        stop_body
    );

    kill_child(child_id);
    let _ = child.wait();
    tokio::fs::remove_file(config_path).await.ok();
}

// ============================================================
// TC-CONF-004: Real-time monitoring via web interface
// Verifies: REQ-REAL-TIME-MONITORING, REQ-WEB-INTERFACE
// ============================================================
#[tokio::test]
async fn tc_conf_004_web_monitoring() {
    let config = r#"
[[tasks]]
type = "Custom"
name = "MonitoredTask"
"#;
    let config_path = "/tmp/tc_conf_004.toml";
    tokio::fs::write(config_path, config).await.unwrap();

    let (mut child, child_id) = start_rsim_background(config_path, "30");
    tokio::time::sleep(Duration::from_secs(3)).await;

    let client = reqwest::Client::new();

    // Test /data endpoint
    let data_resp = client.get("http://127.0.0.1:3030/data")
        .timeout(Duration::from_secs(3))
        .send().await;
    assert!(data_resp.is_ok(), "/data endpoint should be reachable");
    let data_body = data_resp.unwrap().text().await.unwrap_or_default();
    assert!(
        data_body.contains("current_time_secs"),
        "/data should return simulation time. Got: {}",
        &data_body[..data_body.len().min(200)]
    );

    // Test /graph endpoint
    let graph_resp = client.get("http://127.0.0.1:3030/graph")
        .timeout(Duration::from_secs(3))
        .send().await;
    assert!(graph_resp.is_ok(), "/graph endpoint should be reachable");
    let graph_body = graph_resp.unwrap().text().await.unwrap_or_default();
    assert!(
        graph_body.contains("tasks") && graph_body.contains("dependencies"),
        "/graph should return tasks and dependencies JSON. Got: {}",
        &graph_body[..graph_body.len().min(300)]
    );
    assert!(
        graph_body.contains("MonitoredTask"),
        "/graph should include the task name"
    );

    // Test /parameters endpoint
    let params_resp = client.get("http://127.0.0.1:3030/parameters")
        .timeout(Duration::from_secs(3))
        .send().await;
    assert!(params_resp.is_ok(), "/parameters endpoint should be reachable");

    kill_child(child_id);
    let _ = child.wait();
    tokio::fs::remove_file(config_path).await.ok();
}

// ============================================================
// TC-FMU-002: FMU parameter access and tuning via web
// Verifies: REQ-FMU-PARAMETERS, REQ-PARAMETER-TUNING
// ============================================================
#[tokio::test]
async fn tc_fmu_002_fmu_parameter_tuning_via_web() {
    let fmu_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/debug/deps/libfmu_test.dylib");

    if !fmu_path.exists() {
        eprintln!("SKIP: FMU library not found at {:?}", fmu_path);
        return;
    }

    let config = format!(
        r#"
[[tasks]]
type = "Fmu"
name = "TunableFMU"
path = "{}"
"#,
        fmu_path.to_str().unwrap()
    );
    let config_path = "/tmp/tc_fmu_002.toml";
    tokio::fs::write(config_path, &config).await.unwrap();

    let (mut child, child_id) = start_rsim_background(config_path, "30");
    tokio::time::sleep(Duration::from_secs(3)).await;

    let client = reqwest::Client::new();

    // GET current parameters
    let params_resp = client.get("http://127.0.0.1:3030/parameters")
        .timeout(Duration::from_secs(3))
        .send().await;
    assert!(params_resp.is_ok(), "/parameters should be reachable");
    let params_body = params_resp.unwrap().text().await.unwrap_or_default();
    assert!(
        params_body.contains("TunableFMU") && params_body.contains("gain"),
        "Should list FMU parameters including 'gain'. Got: {}",
        params_body
    );

    // POST to set a parameter
    let set_body = serde_json::json!({
        "task_name": "TunableFMU",
        "param_name": "gain",
        "param_value": { "Float": 2.5 }
    });
    let set_resp = client.post("http://127.0.0.1:3030/parameters/set")
        .json(&set_body)
        .timeout(Duration::from_secs(3))
        .send().await;
    assert!(set_resp.is_ok(), "Parameter set should succeed");
    let set_status = set_resp.unwrap().status();
    assert!(set_status.is_success(), "Set parameter should return 200. Got: {}", set_status);

    // Verify the parameter was updated
    let params_resp2 = client.get("http://127.0.0.1:3030/parameters")
        .timeout(Duration::from_secs(3))
        .send().await;
    let params_body2 = params_resp2.unwrap().text().await.unwrap_or_default();
    assert!(
        params_body2.contains("2.5"),
        "Updated gain=2.5 should be reflected. Got: {}",
        params_body2
    );

    kill_child(child_id);
    let _ = child.wait();
    tokio::fs::remove_file(config_path).await.ok();
}

// ============================================================
// TC-FMU-003: Model integration from external environment
// Verifies: REQ-MODEL-INTEGRATION
// ============================================================
#[tokio::test]
async fn tc_fmu_003_model_integration() {
    let fmu_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/debug/deps/libfmu_test.dylib");

    if !fmu_path.exists() {
        eprintln!("SKIP: FMU library not found at {:?}", fmu_path);
        return;
    }

    // The fmu_test library simulates an external model: do_step(t) = t + 1.0
    // This test verifies the framework can load and execute an externally compiled model
    let config = format!(
        r#"
[[tasks]]
type = "Fmu"
name = "ExternalModel"
path = "{}"

[[tasks]]
type = "Custom"
name = "Controller"
"#,
        fmu_path.to_str().unwrap()
    );
    let config_path = PathBuf::from("/tmp/tc_fmu_003.toml");
    tokio::fs::write(&config_path, &config).await.unwrap();

    let (success, stdout, stderr) = run_rsim(&[
        "run", "-c", config_path.to_str().unwrap(), "-s", "1", "-t", "500",
    ])
    .await;

    assert!(
        success,
        "External model integration should succeed. stderr: {}",
        stderr
    );
    assert!(
        stdout.contains("Library loaded successfully"),
        "External model library should load"
    );
    assert!(
        stdout.contains("Executing FMU Task ExternalModel"),
        "External model should execute alongside other tasks"
    );
    assert!(
        stdout.contains("Executing Custom Task Controller"),
        "Controller task should also execute"
    );

    tokio::fs::remove_file(&config_path).await.ok();
}

// ============================================================
// TC-IO-002: Serial task
// Verifies: REQ-IO-SERIAL
// ============================================================
#[tokio::test]
async fn tc_io_002_serial_task() {
    let config = r#"
[[tasks]]
type = "Serial"
name = "Serial_Test"
port = "/dev/ttyUSB0"
baud_rate = 115200
"#;
    let config_path = PathBuf::from("/tmp/tc_io_002.toml");
    tokio::fs::write(&config_path, config).await.unwrap();

    let (success, stdout, _) = run_rsim(&[
        "run", "-c", config_path.to_str().unwrap(), "-s", "1", "-t", "500",
    ])
    .await;

    assert!(success, "Serial simulation should succeed");
    assert!(
        stdout.contains("Serial Task Serial_Test initialized for port /dev/ttyUSB0 at 115200 baud"),
        "Serial task should initialize with correct port and baud rate"
    );
    assert!(
        stdout.contains("Serial Task Serial_Test reading inputs"),
        "Serial task should read inputs"
    );
    assert!(
        stdout.contains("Serial Task Serial_Test writing outputs"),
        "Serial task should write outputs"
    );

    tokio::fs::remove_file(&config_path).await.ok();
}

// ============================================================
// TC-IO-005: Custom task integration and extensibility
// Verifies: REQ-IO-EXTENSIBILITY, REQ-CUSTOM-COMPONENT-INTEGRATION
// ============================================================
#[tokio::test]
async fn tc_io_005_custom_task_extensibility() {
    let config = r#"
[[tasks]]
type = "Custom"
name = "ExtensibleTask"
"#;
    let config_path = PathBuf::from("/tmp/tc_io_005.toml");
    tokio::fs::write(&config_path, config).await.unwrap();

    let (success, stdout, _) = run_rsim(&[
        "run", "-c", config_path.to_str().unwrap(), "-s", "1", "-t", "500",
    ])
    .await;

    assert!(success, "Custom task should succeed");
    assert!(
        stdout.contains("Creating task: Custom(CustomConfig { name: \"ExtensibleTask\" })"),
        "TaskFactory should create the custom task"
    );
    assert!(
        stdout.contains("Executing Custom Task ExtensibleTask"),
        "Custom task should execute with its logic"
    );

    tokio::fs::remove_file(&config_path).await.ok();
}

// ============================================================
// TC-IO-007: Synchronized I/O — multiple I/O tasks in same time step
// Verifies: REQ-IO-SYNCHRONIZATION
// ============================================================
#[tokio::test]
async fn tc_io_007_synchronized_io() {
    let config = r#"
[[tasks]]
type = "Gpio"
name = "GPIO_Sync"
pins = [4, 5]

[[tasks]]
type = "Udp"
name = "UDP_Sync"
local_addr = "127.0.0.1:19090"
remote_addr = "127.0.0.1:19091"

[[tasks]]
type = "Analog"
name = "Analog_Sync"
channels = [0, 1]
is_input = true
"#;
    let config_path = PathBuf::from("/tmp/tc_io_007.toml");
    tokio::fs::write(&config_path, config).await.unwrap();

    let (success, stdout, _) = run_rsim(&[
        "run", "-c", config_path.to_str().unwrap(), "-s", "1", "-t", "500",
    ])
    .await;

    assert!(success, "Multi-I/O simulation should succeed");

    // Verify all I/O tasks are created
    assert!(stdout.contains("GPIO_Sync"), "GPIO task should be created");
    assert!(stdout.contains("UDP_Sync"), "UDP task should be created");
    assert!(stdout.contains("Analog_Sync"), "Analog task should be created");

    // Verify I/O tasks are initialized
    assert!(
        stdout.contains("Initializing I/O for task: GPIO_Sync"),
        "GPIO should be initialized as I/O task"
    );
    assert!(
        stdout.contains("Initializing I/O for task: UDP_Sync"),
        "UDP should be initialized as I/O task"
    );

    // Verify all I/O tasks are read/written in same time step
    assert!(
        stdout.contains("Reading I/O for task: GPIO_Sync"),
        "GPIO should read I/O each step"
    );
    assert!(
        stdout.contains("Writing I/O for task: GPIO_Sync"),
        "GPIO should write I/O each step"
    );

    tokio::fs::remove_file(&config_path).await.ok();
}

// ============================================================
// TC-IO-008: High-speed data acquisition (Analog with sampling_rate_hz)
// Verifies: REQ-IO-HIGH-SPEED-DAQ
// ============================================================
#[tokio::test]
async fn tc_io_008_high_speed_daq() {
    let config = r#"
[[tasks]]
type = "Analog"
name = "HighSpeedDAQ"
channels = [0, 1, 2, 3]
is_input = true
sampling_rate_hz = 10000
"#;
    let config_path = PathBuf::from("/tmp/tc_io_008.toml");
    tokio::fs::write(&config_path, config).await.unwrap();

    let (success, stdout, _) = run_rsim(&[
        "run", "-c", config_path.to_str().unwrap(), "-s", "1", "-t", "500",
    ])
    .await;

    assert!(success, "High-speed DAQ config should be accepted");
    assert!(
        stdout.contains("Creating task: Analog"),
        "Analog task with sampling_rate_hz should be created"
    );
    assert!(
        stdout.contains("HighSpeedDAQ"),
        "Task name should appear in output"
    );

    tokio::fs::remove_file(&config_path).await.ok();
}