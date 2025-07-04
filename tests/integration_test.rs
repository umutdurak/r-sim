use std::process::{Command, Stdio};
use std::path::PathBuf;
use tokio::fs;
use tokio::time::{timeout, Duration};
use std::io::{BufReader, Read};

#[tokio::test]
async fn test_framework_execution() {
    // Create dummy config file
    let fmu_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fmu_test/target/debug/libfmu_test.so");
    let config_content = r#"
tasks = []
dependencies = []
time_multiplier = 1.0
[logging]
log_file = "test_simulation_log.csv"
log_interval_millis = 100
logged_outputs = { }
"#.to_string();
    let config_path = PathBuf::from("test_config.toml");
    tokio::fs::write(&config_path, config_content).await.unwrap();

    // Spawn the r-sim executable as a subprocess
    let mut child = Command::new("target/debug/r-sim")
        .arg("run")
        .arg("--config-file")
        .arg(&config_path)
        .arg("--simulation-duration-secs")
        .arg("1")
        .arg("--time-step-millis")
        .arg("100")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn r-sim process");

    let child_id = child.id();

    let stdout = child.stdout.take().expect("Failed to take stdout");
    let stderr = child.stderr.take().expect("Failed to take stderr");

    let stdout_task = tokio::task::spawn_blocking(move || {
        let mut reader = BufReader::new(stdout);
        let mut buffer = String::new();
        reader.read_to_string(&mut buffer).expect("Failed to read stdout");
        buffer
    });

    let stderr_task = tokio::task::spawn_blocking(move || {
        let mut reader = BufReader::new(stderr);
        let mut buffer = String::new();
        reader.read_to_string(&mut buffer).expect("Failed to read stderr");
        buffer
    });

    let run_result = tokio::select! {
        output = timeout(Duration::from_secs(5), tokio::task::spawn_blocking(move || child.wait())) => {
            match output {
                Ok(Ok(status)) => {
                    let stdout_output = stdout_task.await.expect("Failed to join stdout task");
                    let stderr_output = stderr_task.await.expect("Failed to join stderr task");
                    (status, stdout_output, stderr_output)
                },
                Ok(Err(e)) => panic!("Failed to wait for child process: {:?}", e),
                Err(_) => {
                    // Timeout occurred, kill the child process
                    Command::new("kill").arg(child_id.to_string()).output().expect("Failed to kill process");
                    let stdout_output = stdout_task.await.expect("Failed to join stdout task after kill");
                    let stderr_output = stderr_task.await.expect("Failed to join stderr task after kill");
                    panic!("r-sim process timed out and was killed. PID: {}\nStderr: {}\nStdout: {}", child_id, stderr_output, stdout_output);
                }
            }
        }
    };

    // Assert that the r-sim process exited successfully
    let exit_status = run_result.0.expect("Failed to get exit status");
    assert!(exit_status.success(), "r-sim process failed: {:?}\nStderr: {}\nStdout: {}", exit_status, run_result.2, run_result.1);

    // Clean up dummy config file
    tokio::fs::remove_file(&config_path).await.unwrap();
}