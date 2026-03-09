use std::path::PathBuf;
use tokio::fs;
use tokio::time::{self, Duration, timeout};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::toposort;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use csv::Writer;
use std::fs::File;
use std::any::Any;
use warp::Filter;
use tokio::sync::{watch, RwLock};
use std::sync::Arc;
use thiserror::Error;
use async_trait::async_trait;
use libloading::{Library, Symbol};

#[derive(Error, Debug)]
pub enum FrameworkError {
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("Graph error: {0}")]
    GraphError(String),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("CSV error: {0}")]
    CsvError(#[from] csv::Error),
    #[error("TOML deserialization error: {0}")]
    TomlError(#[from] toml::de::Error),
    #[error("Task execution error: {0}")]
    TaskExecutionError(String),
    #[error("Web server error: {0}")]
    WebServerError(String),
    #[error("Libloading error: {0}")]
    LibloadingError(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Represents the current status of the simulation.
#[derive(Debug, Clone, PartialEq)]
pub enum SimulationStatus {
    Stopped,
    Running,
    Paused,
}

/// Represents the data to be monitored in real-time.
#[derive(Debug, Clone, Serialize)]
pub struct SimulationData {
    pub current_time_secs: f64,
    pub task_outputs: HashMap<String, HashMap<String, f64>>,
    pub task_execution_times_micros: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WebTaskInfo {
    pub name: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WebDependencyInfo {
    pub from_task: String,
    pub to_task: String,
    pub data_flow: String,
    pub dep_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WebGraphInfo {
    pub tasks: Vec<WebTaskInfo>,
    pub dependencies: Vec<WebDependencyInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Parameter {
    Float(f64),
    Integer(i64),
    Boolean(bool),
    String(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetParameterRequest {
    pub task_name: String,
    pub param_name: String,
    pub param_value: Parameter,
}

/// Defines the interface for any simulation task within the framework.
/// Custom components should implement this trait to be integrated into the simulation graph.
#[async_trait]
pub trait SimulationTask: Send + Sync + 'static {
    /// Executes the task for the current simulation time step.
    /// Implementations should perform their core logic here, such as updating internal states,
    /// processing inputs, and producing outputs.
    async fn execute(&mut self, current_time: Duration) -> Result<(), FrameworkError>;
    /// Returns a list of input variable names that this task expects.
    fn get_inputs(&self) -> Vec<String>;
    /// Returns a list of output variable names that this task produces.
    fn get_outputs(&self) -> Vec<String>;
    /// Sets the value of a specific input variable.
    ///
    /// # Arguments
    /// * `name` - The name of the input variable.
    /// * `value` - The value to set for the input variable.
    fn set_input(&mut self, name: &str, value: f64);
    /// Retrieves the current value of a specific output variable.
    ///
    /// # Arguments
    /// * `name` - The name of the output variable.
    ///
    /// # Returns
    /// An `Option<f64>` containing the value if the output variable exists, otherwise `None`.
    fn get_output_value(&self, name: &str) -> Option<f64>;
    /// Retrieves the current value of a specific input variable.
    ///
    /// # Arguments
    /// * `name` - The name of the input variable.
    ///
    /// # Returns
    /// An `Option<f64>` containing the value if the input variable exists, otherwise `None`.
    fn get_input_value(&self, name: &str) -> Option<f64>;
    /// Provides a way to downcast the trait object to a concrete type.
    /// This is useful for accessing specific fields or methods of a concrete task implementation.
    fn as_any(&self) -> &dyn Any;
    /// Provides a mutable way to downcast the trait object to a concrete type.
    /// This is useful for accessing and modifying specific fields or methods of a concrete task implementation.
    fn as_any_mut(&mut self) -> &mut dyn Any;
    /// Returns the name of the task.
    fn get_name(&self) -> String;
    /// Returns a map of parameter names to their current values.
    fn get_parameters(&self) -> HashMap<String, Parameter>;
    /// Sets the value of a specific parameter.
    fn set_parameter(&mut self, name: &str, value: Parameter) -> Result<(), FrameworkError>;
}

// Type alias for a function that creates a Box<dyn SimulationTask> from a TaskConfig
pub type TaskCreator = fn(TaskConfig) -> Result<Box<dyn SimulationTask>, FrameworkError>;

pub struct TaskFactory {
    creators: HashMap<String, TaskCreator>,
}

impl TaskFactory {
    pub fn new() -> Self {
        TaskFactory {
            creators: HashMap::new(),
        }
    }

    pub fn register_task(&mut self, task_type: &'static str, creator: TaskCreator) {
        self.creators.insert(task_type.to_string(), creator);
    }

    pub fn create_task(&self, config: TaskConfig) -> Result<Box<dyn SimulationTask>, FrameworkError> {
        let task_type_str = match &config {
            TaskConfig::Fmu(_) => "Fmu",
            TaskConfig::Gpio(_) => "Gpio",
            TaskConfig::Serial(_) => "Serial",
            TaskConfig::Udp(_) => "Udp",
            TaskConfig::Analog(_) => "Analog",
            TaskConfig::ModbusTcp(_) => "ModbusTcp",
            TaskConfig::Custom(_) => "Custom",
        };

        if let Some(creator) = self.creators.get(task_type_str) {
            creator(config)
        } else {
            Err(FrameworkError::ConfigurationError(format!("Unknown task type: {}", task_type_str)))
        }
    }
}

pub struct FmuTask {
    name: String,
    inputs: HashMap<String, f64>,
    outputs: HashMap<String, f64>,
    _library: Library, // Store the library to keep it in scope
    do_step_fn: Symbol<'static, unsafe extern "C" fn(f64) -> f64>,
    parameters: HashMap<String, Parameter>,
}

impl FmuTask {
    pub fn new(name: String, path: String) -> Result<Self, FrameworkError> {
        println!("FmuTask::new: Attempting to load library from path: {}", path);
        let library = unsafe { Library::new(path.clone()).map_err(|e| FrameworkError::LibloadingError(e.to_string()))? };
        println!("FmuTask::new: Library loaded successfully.");
        let do_step_fn_raw: Symbol<unsafe extern "C" fn(f64) -> f64> = unsafe { 
            library.get(b"do_step").map_err(|e| FrameworkError::LibloadingError(e.to_string()))?
        };

        // SAFETY: This transmute is safe because `library` is moved into `self`,
        // ensuring it lives as long as `do_step_fn`.
        let do_step_fn = unsafe { std::mem::transmute(do_step_fn_raw) };

        let mut inputs = HashMap::new();
        inputs.insert("input_var".to_string(), 0.0);

        let mut outputs = HashMap::new();
        outputs.insert("output_var".to_string(), 0.0);

        let mut parameters = HashMap::new();
        parameters.insert("gain".to_string(), Parameter::Float(1.0));

        Ok(FmuTask { 
            name,
            inputs,
            outputs,
            _library: library,
            do_step_fn,
            parameters,
        })
    }
}

#[async_trait]
impl SimulationTask for FmuTask {
    async fn execute(&mut self, current_time: Duration) -> Result<(), FrameworkError> {
        println!("  Executing FMU Task {}: {:?}", self.name, current_time);
        let output = unsafe { (self.do_step_fn)(current_time.as_secs_f64()) };
        if let Some(output_val) = self.outputs.get_mut("output_var") {
            *output_val = output;
        }
        Ok(())
    }

    fn get_inputs(&self) -> Vec<String> {
        self.inputs.keys().cloned().collect()
    }

    fn get_outputs(&self) -> Vec<String> {
        self.outputs.keys().cloned().collect()
    }

    fn set_input(&mut self, name: &str, value: f64) {
        self.inputs.insert(name.to_string(), value);
    }

    fn get_output_value(&self, name: &str) -> Option<f64> {
        self.outputs.get(name).cloned()
    }

    fn get_input_value(&self, name: &str) -> Option<f64> {
        self.inputs.get(name).cloned()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_parameters(&self) -> HashMap<String, Parameter> {
        self.parameters.clone()
    }

    fn set_parameter(&mut self, name: &str, value: Parameter) -> Result<(), FrameworkError> {
        self.parameters.insert(name.to_string(), value);
        Ok(())
    }
}

// Define a trait for I/O tasks
#[async_trait]
pub trait IoTask: SimulationTask {
    // Specific I/O methods can be added here later
    async fn initialize_io(&mut self) -> Result<(), FrameworkError>;
    async fn read_io(&mut self) -> Result<(), FrameworkError>;
    async fn write_io(&mut self) -> Result<(), FrameworkError>;
}

// Placeholder for GPIO Task
pub struct GpioTask {
    name: String,
    inputs: HashMap<String, f64>,
    outputs: HashMap<String, f64>,
    pins: Vec<u8>,
    parameters: HashMap<String, Parameter>,
}

impl GpioTask {
    pub fn new(name: String, pins: Vec<u8>) -> Self {
        let mut parameters = HashMap::new();
        parameters.insert("pins".to_string(), Parameter::String(format!("{:?}", pins)));
        GpioTask {
            name,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            pins,
            parameters,
        }
    }
}

#[async_trait]
impl SimulationTask for GpioTask {
    async fn execute(&mut self, _current_time: Duration) -> Result<(), FrameworkError> {
        println!("  Executing GPIO Task {}", self.name);
        // Call trait methods explicitly
        let _ = <Self as IoTask>::read_io(self).await?;
        // Process inputs if any
        let _ = <Self as IoTask>::write_io(self).await?;
        Ok(())
    }

    fn get_inputs(&self) -> Vec<String> {
        self.inputs.keys().cloned().collect()
    }

    fn get_outputs(&self) -> Vec<String> {
        self.outputs.keys().cloned().collect()
    }

    fn set_input(&mut self, name: &str, value: f64) {
        self.inputs.insert(name.to_string(), value);
    }

    fn get_output_value(&self, name: &str) -> Option<f64> {
        self.outputs.get(name).cloned()
    }

    fn get_input_value(&self, name: &str) -> Option<f64> {
        self.inputs.get(name).cloned()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_parameters(&self) -> HashMap<String, Parameter> {
        self.parameters.clone()
    }

    fn set_parameter(&mut self, name: &str, value: Parameter) -> Result<(), FrameworkError> {
        match name {
            "pins" => {
                if let Parameter::String(s) = value {
                    // Attempt to parse the string back into a Vec<u8>
                    // This is a simplified parsing and might need more robust error handling
                    let parsed_pins: Result<Vec<u8>, _> = s.trim_matches(|c| c == '[' || c == ']')
                                                          .split(',')
                                                          .map(|s| s.trim().parse::<u8>())
                                                          .collect();
                    match parsed_pins {
                        Ok(pins) => {
                            self.pins = pins;
                            self.parameters.insert(name.to_string(), Parameter::String(s));
                            Ok(())
                        },
                        Err(_) => Err(FrameworkError::TaskExecutionError(format!("Invalid value for pins parameter: {}", s))),
                    }
                } else {
                    Err(FrameworkError::TaskExecutionError("Pins parameter expects a string value.".to_string()))
                }
            },
            _ => Err(FrameworkError::TaskExecutionError(format!("Parameter '{}' not found for GPIO task '{}'.", name, self.name))),
        }
    }
}

#[async_trait]
impl IoTask for GpioTask {
    async fn initialize_io(&mut self) -> Result<(), FrameworkError> {
        println!("    GPIO Task {} initialized for pins: {:?}", self.name, self.pins);
        Ok(())
    }
    async fn read_io(&mut self) -> Result<(), FrameworkError> {
        println!("    GPIO Task {} reading inputs from pins: {:?}", self.name, self.pins);
        // Simulate reading GPIO pins
        Ok(())
    }
    async fn write_io(&mut self) -> Result<(), FrameworkError> {
        println!("    GPIO Task {} writing outputs to pins: {:?}", self.name, self.pins);
        // Simulate writing to GPIO pins
        Ok(())
    }
}

// Placeholder for Serial Communication Task
pub struct SerialTask {
    name: String,
    inputs: HashMap<String, f64>,
    outputs: HashMap<String, f64>,
    port: String,
    baud_rate: u32,
    parameters: HashMap<String, Parameter>,
}

impl SerialTask {
    pub fn new(name: String, port: String, baud_rate: u32) -> Self {
        let mut parameters = HashMap::new();
        parameters.insert("port".to_string(), Parameter::String(port.clone()));
        parameters.insert("baud_rate".to_string(), Parameter::Integer(baud_rate as i64));
        SerialTask {
            name,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            port,
            baud_rate,
            parameters,
        }
    }
}

#[async_trait]
impl SimulationTask for SerialTask {
    async fn execute(&mut self, _current_time: Duration) -> Result<(), FrameworkError> {
        println!("  Executing Serial Task {}", self.name);
        let _ = <Self as IoTask>::read_io(self).await?;
        // Process inputs if any
        let _ = <Self as IoTask>::write_io(self).await?;
        Ok(())
    }

    fn get_inputs(&self) -> Vec<String> {
        self.inputs.keys().cloned().collect()
    }

    fn get_outputs(&self) -> Vec<String> {
        self.outputs.keys().cloned().collect()
    }

    fn set_input(&mut self, name: &str, value: f64) {
        self.inputs.insert(name.to_string(), value);
    }

    fn get_output_value(&self, name: &str) -> Option<f64> {
        self.outputs.get(name).cloned()
    }

    fn get_input_value(&self, name: &str) -> Option<f64> {
        self.inputs.get(name).cloned()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_parameters(&self) -> HashMap<String, Parameter> {
        self.parameters.clone()
    }

    fn set_parameter(&mut self, name: &str, value: Parameter) -> Result<(), FrameworkError> {
        match name {
            "port" => {
                if let Parameter::String(s) = value {
                    self.port = s.clone();
                    self.parameters.insert(name.to_string(), Parameter::String(s));
                    Ok(())
                } else {
                    Err(FrameworkError::TaskExecutionError("Port parameter expects a string value.".to_string()))
                }
            },
            "baud_rate" => {
                if let Parameter::Integer(i) = value {
                    self.baud_rate = i as u32;
                    self.parameters.insert(name.to_string(), Parameter::Integer(i));
                    Ok(())
                } else {
                    Err(FrameworkError::TaskExecutionError("Baud rate parameter expects an integer value.".to_string()))
                }
            },
            _ => Err(FrameworkError::TaskExecutionError(format!("Parameter '{}' not found for Serial task '{}'.", name, self.name))),
        }
    }
}

#[async_trait]
impl IoTask for SerialTask {
    async fn initialize_io(&mut self) -> Result<(), FrameworkError> {
        println!("    Serial Task {} initialized for port {} at {} baud.", self.name, self.port, self.baud_rate);
        Ok(())
    }
    async fn read_io(&mut self) -> Result<(), FrameworkError> {
        println!("    Serial Task {} reading inputs from port {}.", self.name, self.port);
        // Simulate reading serial data
        Ok(())
    }
    async fn write_io(&mut self) -> Result<(), FrameworkError> {
        println!("    Serial Task {} writing outputs to port {}.", self.name, self.port);
        // Simulate writing serial data
        Ok(())
    }
}

// Placeholder for UDP Communication Task
pub struct UdpTask {
    name: String,
    inputs: HashMap<String, f64>,
    outputs: HashMap<String, f64>,
    socket: Option<tokio::net::UdpSocket>,
    local_addr: String,
    remote_addr: String,
    parameters: HashMap<String, Parameter>,
}

impl UdpTask {
    pub fn new(name: String, local_addr: String, remote_addr: String) -> Self {
        let mut parameters = HashMap::new();
        parameters.insert("local_addr".to_string(), Parameter::String(local_addr.clone()));
        parameters.insert("remote_addr".to_string(), Parameter::String(remote_addr.clone()));
        UdpTask {
            name,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            socket: None,
            local_addr,
            remote_addr,
            parameters,
        }
    }
}

#[async_trait]
impl SimulationTask for UdpTask {
    async fn execute(&mut self, _current_time: Duration) -> Result<(), FrameworkError> {
        println!("  Executing UDP Task {}", self.name);
        let _ = <Self as IoTask>::read_io(self).await?;
        // Process inputs if any
        let _ = <Self as IoTask>::write_io(self).await?;
        Ok(())
    }

    fn get_inputs(&self) -> Vec<String> {
        self.inputs.keys().cloned().collect()
    }

    fn get_outputs(&self) -> Vec<String> {
        self.outputs.keys().cloned().collect()
    }

    fn set_input(&mut self, name: &str, value: f64) {
        self.inputs.insert(name.to_string(), value);
    }

    fn get_output_value(&self, name: &str) -> Option<f64> {
        self.outputs.get(name).cloned()
    }

    fn get_input_value(&self, name: &str) -> Option<f64> {
        self.inputs.get(name).cloned()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_parameters(&self) -> HashMap<String, Parameter> {
        self.parameters.clone()
    }

    fn set_parameter(&mut self, name: &str, value: Parameter) -> Result<(), FrameworkError> {
        match name {
            "local_addr" => {
                if let Parameter::String(s) = value {
                    self.local_addr = s.clone();
                    self.parameters.insert(name.to_string(), Parameter::String(s));
                    Ok(())
                } else {
                    Err(FrameworkError::TaskExecutionError("Local address parameter expects a string value.".to_string()))
                }
            },
            "remote_addr" => {
                if let Parameter::String(s) = value {
                    self.remote_addr = s.clone();
                    self.parameters.insert(name.to_string(), Parameter::String(s));
                    Ok(())
                } else {
                    Err(FrameworkError::TaskExecutionError("Remote address parameter expects a string value.".to_string()))
                }
            },
            _ => Err(FrameworkError::TaskExecutionError(format!("Parameter '{}' not found for UDP task '{}'.", name, self.name))),
        }
    }
}

#[async_trait]
impl IoTask for UdpTask {
    async fn initialize_io(&mut self) -> Result<(), FrameworkError> {
        let socket = tokio::net::UdpSocket::bind(&self.local_addr).await.map_err(|e| FrameworkError::IoError(e))?;
        socket.connect(&self.remote_addr).await.map_err(|e| FrameworkError::IoError(e))?;
        self.socket = Some(socket);
        println!("    UDP Task {} initialized and bound to {}.", self.name, self.local_addr);
        Ok(())
    }

    async fn read_io(&mut self) -> Result<(), FrameworkError> {
        if let Some(socket) = &self.socket {
            let mut buf = [0; 1024];
            match socket.try_recv(&mut buf) {
                Ok(len) => {
                    let received_data = &buf[..len];
                    println!("    UDP Task {} received {} bytes: {:?}", self.name, len, received_data);
                    // Assuming the received data is a f64 for now
                    if len >= 8 {
                        let value = f64::from_be_bytes(buf[..8].try_into().unwrap());
                        self.outputs.insert("udp_output".to_string(), value);
                    }
                },
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No data available to read
                },
                Err(e) => {
                    return Err(FrameworkError::IoError(e));
                }
            }
        }
        Ok(())
    }

    async fn write_io(&mut self) -> Result<(), FrameworkError> {
        if let Some(value) = self.get_input_value("udp_input") {
            let data = value.to_be_bytes();
            match self.socket.as_ref().unwrap().send(&data).await {
                Ok(_) => {
                    println!("    UDP Task {} sent value: {}", self.name, value);
                },
                Err(e) => {
                    return Err(FrameworkError::IoError(e));
                }
            }
        }
        Ok(())
    }
}

/// Placeholder for Analog Input/Output Task.
pub struct AnalogTask {
    name: String,
    inputs: HashMap<String, f64>,
    outputs: HashMap<String, f64>,
    channels: Vec<u8>,
    is_input: bool,
    sampling_rate_hz: Option<u32>,
    parameters: HashMap<String, Parameter>,
}

impl AnalogTask {
    pub fn new(name: String, channels: Vec<u8>, is_input: bool, sampling_rate_hz: Option<u32>) -> Self {
        let mut parameters = HashMap::new();
        parameters.insert("channels".to_string(), Parameter::String(format!("{:?}", channels)));
        parameters.insert("is_input".to_string(), Parameter::Boolean(is_input));
        if let Some(rate) = sampling_rate_hz {
            parameters.insert("sampling_rate_hz".to_string(), Parameter::Integer(rate as i64));
        }
        AnalogTask {
            name,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            channels,
            is_input,
            sampling_rate_hz,
            parameters,
        }
    }
}

#[async_trait]
impl SimulationTask for AnalogTask {
    async fn execute(&mut self, _current_time: Duration) -> Result<(), FrameworkError> {
        println!("  Executing Analog Task {}", self.name);
        if self.is_input {
            let _ = <Self as IoTask>::read_io(self).await?;
        } else {
            let _ = <Self as IoTask>::write_io(self).await?;
        }
        Ok(())
    }

    fn get_inputs(&self) -> Vec<String> {
        self.inputs.keys().cloned().collect()
    }

    fn get_outputs(&self) -> Vec<String> {
        self.outputs.keys().cloned().collect()
    }

    fn set_input(&mut self, name: &str, value: f64) {
        self.inputs.insert(name.to_string(), value);
    }

    fn get_output_value(&self, name: &str) -> Option<f64> {
        self.outputs.get(name).cloned()
    }

    fn get_input_value(&self, name: &str) -> Option<f64> {
        self.inputs.get(name).cloned()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_parameters(&self) -> HashMap<String, Parameter> {
        self.parameters.clone()
    }

    fn set_parameter(&mut self, name: &str, value: Parameter) -> Result<(), FrameworkError> {
        match name {
            "channels" => {
                if let Parameter::String(s) = value {
                    // Attempt to parse the string back into a Vec<u8>
                    // This is a simplified parsing and might need more robust error handling
                    let parsed_channels: Result<Vec<u8>, _> = s.trim_matches(|c| c == '[' || c == ']')
                                                              .split(',')
                                                              .filter(|s| !s.is_empty())
                                                              .map(|s| s.trim().parse::<u8>())
                                                              .collect();
                    match parsed_channels {
                        Ok(channels) => {
                            self.channels = channels;
                            self.parameters.insert(name.to_string(), Parameter::String(s));
                            Ok(())
                        },
                        Err(_) => Err(FrameworkError::TaskExecutionError(format!("Invalid value for channels parameter: {}", s))),
                    }
                } else {
                    Err(FrameworkError::TaskExecutionError("Channels parameter expects a string value.".to_string()))
                }
            },
            "is_input" => {
                if let Parameter::Boolean(b) = value {
                    self.is_input = b;
                    self.parameters.insert(name.to_string(), Parameter::Boolean(b));
                    Ok(())
                } else {
                    Err(FrameworkError::TaskExecutionError("Is input parameter expects a boolean value.".to_string()))
                }
            },
            "sampling_rate_hz" => {
                if let Parameter::Integer(i) = value {
                    self.sampling_rate_hz = Some(i as u32);
                    self.parameters.insert(name.to_string(), Parameter::Integer(i));
                    Ok(())
                } else if let Parameter::Float(f) = value {
                    self.sampling_rate_hz = Some(f as u32);
                    self.parameters.insert(name.to_string(), Parameter::Float(f));
                    Ok(())
                } else {
                    Err(FrameworkError::TaskExecutionError("Sampling rate parameter expects an integer or float value.".to_string()))
                }
            },
            _ => Err(FrameworkError::TaskExecutionError(format!("Parameter '{}' not found for Analog task '{}'.", name, self.name))),
        }
    }
}

#[async_trait]
impl IoTask for AnalogTask {
    async fn initialize_io(&mut self) -> Result<(), FrameworkError> {
        println!("    Analog Task {} initialized for channels: {:?}. Is input: {}.", self.name, self.channels, self.is_input);
        Ok(())
    }
    async fn read_io(&mut self) -> Result<(), FrameworkError> {
        println!("    Analog Task {} reading inputs from channels: {:?}", self.name, self.channels);
        // Simulate reading analog data
        Ok(())
    }
    async fn write_io(&mut self) -> Result<(), FrameworkError> {
        println!("    Analog Task {} writing outputs to channels: {:?}", self.name, self.channels);
        // Simulate writing analog data
        Ok(())
    }
}

/// Placeholder for Modbus TCP Task.
pub struct ModbusTcpTask {
    name: String,
    inputs: HashMap<String, f64>,
    outputs: HashMap<String, f64>,
    ip_address: String,
    port: u16,
    parameters: HashMap<String, Parameter>,
}

impl ModbusTcpTask {
    pub fn new(name: String, ip_address: String, port: u16) -> Self {
        let mut parameters = HashMap::new();
        parameters.insert("ip_address".to_string(), Parameter::String(ip_address.clone()));
        parameters.insert("port".to_string(), Parameter::Integer(port as i64));
        ModbusTcpTask {
            name,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            ip_address,
            port,
            parameters,
        }
    }
}

#[async_trait]
impl SimulationTask for ModbusTcpTask {
    async fn execute(&mut self, _current_time: Duration) -> Result<(), FrameworkError> {
        println!("  Executing Modbus TCP Task {}", self.name);
        let _ = <Self as IoTask>::read_io(self).await?;
        let _ = <Self as IoTask>::write_io(self).await?;
        Ok(())
    }

    fn get_inputs(&self) -> Vec<String> {
        self.inputs.keys().cloned().collect()
    }

    fn get_outputs(&self) -> Vec<String> {
        self.outputs.keys().cloned().collect()
    }

    fn set_input(&mut self, name: &str, value: f64) {
        self.inputs.insert(name.to_string(), value);
    }

    fn get_output_value(&self, name: &str) -> Option<f64> {
        self.outputs.get(name).cloned()
    }

    fn get_input_value(&self, name: &str) -> Option<f64> {
        self.inputs.get(name).cloned()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_parameters(&self) -> HashMap<String, Parameter> {
        self.parameters.clone()
    }

    fn set_parameter(&mut self, name: &str, value: Parameter) -> Result<(), FrameworkError> {
        match name {
            "ip_address" => {
                if let Parameter::String(s) = value {
                    self.ip_address = s.clone();
                    self.parameters.insert(name.to_string(), Parameter::String(s));
                    Ok(())
                } else {
                    Err(FrameworkError::TaskExecutionError("IP address parameter expects a string value.".to_string()))
                }
            },
            "port" => {
                if let Parameter::Integer(i) = value {
                    self.port = i as u16;
                    self.parameters.insert(name.to_string(), Parameter::Integer(i));
                    Ok(())
                } else {
                    Err(FrameworkError::TaskExecutionError("Port parameter expects an integer value.".to_string()))
                }
            },
            _ => Err(FrameworkError::TaskExecutionError(format!("Parameter '{}' not found for Modbus TCP task '{}'.", name, self.name))),
        }
    }
}

#[async_trait]
impl IoTask for ModbusTcpTask {
    async fn initialize_io(&mut self) -> Result<(), FrameworkError> {
        println!("    Modbus TCP Task {} initialized for {}:{}.", self.name, self.ip_address, self.port);
        Ok(())
    }
    async fn read_io(&mut self) -> Result<(), FrameworkError> {
        println!("    Modbus TCP Task {} reading data.", self.name);
        Ok(())
    }
    async fn write_io(&mut self) -> Result<(), FrameworkError> {
        println!("    Modbus TCP Task {} writing data.", self.name);
        Ok(())
    }
}

/// Placeholder for a custom user-defined task.
pub struct CustomTask {
    name: String,
    inputs: HashMap<String, f64>,
    outputs: HashMap<String, f64>,
    parameters: HashMap<String, Parameter>,
    // Add custom fields here
}

impl CustomTask {
    pub fn new(name: String) -> Self {
        CustomTask {
            name,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            parameters: HashMap::new(),
            // Initialize custom fields
        }
    }
}

#[async_trait]
impl SimulationTask for CustomTask {
    async fn execute(&mut self, current_time: Duration) -> Result<(), FrameworkError> {
        println!("  Executing Custom Task {}: {:?}", self.name, current_time);
        // Implement custom logic here
        // For example, update outputs based on inputs and current_time
        if let Some(output_val) = self.outputs.get_mut("custom_output") {
            *output_val = current_time.as_secs_f64() * 2.0;
        }
        Ok(())
    }

    fn get_inputs(&self) -> Vec<String> {
        self.inputs.keys().cloned().collect()
    }

    fn get_outputs(&self) -> Vec<String> {
        self.outputs.keys().cloned().collect()
    }

    fn set_input(&mut self, name: &str, value: f64) {
        self.inputs.insert(name.to_string(), value);
    }

    fn get_output_value(&self, name: &str) -> Option<f64> {
        self.outputs.get(name).cloned()
    }

    fn get_input_value(&self, name: &str) -> Option<f64> {
        self.inputs.get(name).cloned()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_parameters(&self) -> HashMap<String, Parameter> {
        self.parameters.clone()
    }

    fn set_parameter(&mut self, name: &str, value: Parameter) -> Result<(), FrameworkError> {
        self.parameters.insert(name.to_string(), value);
        Ok(())
    }
}

impl warp::reject::Reject for FrameworkError {}

// Represents the type of dependency between tasks
pub enum DependencyType {
    Direct(String), // Direct data flow, e.g., "output_to_input"
    MemoryBlock(String), // Introduces a one-step delay, breaking causality
}

// Represents the simulation graph with tasks and their dependencies
pub struct SimulationGraph {
    graph: DiGraph<Arc<RwLock<Box<dyn SimulationTask>>>, DependencyType>,
    task_indices: HashMap<String, NodeIndex>,
}

impl SimulationGraph {
    pub fn new() -> Self {
        SimulationGraph {
            graph: DiGraph::new(),
            task_indices: HashMap::new(),
        }
    }

    pub fn add_task(&mut self, name: String, task: Arc<RwLock<Box<dyn SimulationTask>>>) -> NodeIndex {
        let index = self.graph.add_node(task);
        self.task_indices.insert(name, index);
        index
    }

    pub fn add_dependency(&mut self, from_task_name: &str, to_task_name: &str, dep_type: DependencyType) -> Result<(), FrameworkError> {
        let &from_index = self.task_indices.get(from_task_name).ok_or(FrameworkError::GraphError(format!("Task {} not found", from_task_name)))?;
        let &to_index = self.task_indices.get(to_task_name).ok_or(FrameworkError::GraphError(format!("Task {} not found", to_task_name)))?;
        self.graph.add_edge(from_index, to_index, dep_type);
        Ok(())
    }

    pub fn get_execution_order(&self) -> Result<Vec<NodeIndex>, FrameworkError> {
        // Create a graph for topological sort that ignores memory block dependencies
        let mut graph_for_toposort = DiGraph::<(), ()>::new();
        for _node_idx in self.graph.node_indices() {
            graph_for_toposort.add_node(());
        }

        for edge_idx in self.graph.edge_indices() {
            let (u, v) = self.graph.edge_endpoints(edge_idx).unwrap();
            match &self.graph[edge_idx] {
                DependencyType::Direct(_) => {
                    graph_for_toposort.add_edge(u, v, ());
                },
                DependencyType::MemoryBlock(_) => {
                    // Ignore memory block dependencies for topological sort
                }
            }
        }

        match toposort(&graph_for_toposort, None) {
            Ok(order) => Ok(order),
            Err(e) => Err(FrameworkError::GraphError(format!("Causal loop detected that cannot be resolved by memory blocks: {:?}", e.node_id()))),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct FmuConfig {
    name: String,
    path: String,
    // Add other FMU specific configurations here
}

#[derive(Debug, Deserialize)]
pub struct GpioConfig {
    name: String,
    pins: Vec<u8>,
    // Add other GPIO specific configurations here
}

#[derive(Debug, Deserialize)]
pub struct SerialConfig {
    name: String,
    port: String,
    baud_rate: u32,
    // Add other Serial specific configurations here
}

#[derive(Debug, Deserialize)]
pub struct UdpConfig {
    name: String,
    local_addr: String,
    remote_addr: String,
    // Add other UDP specific configurations here
}

#[derive(Debug, Deserialize)]
pub struct AnalogConfig {
    name: String,
    channels: Vec<u8>,
    is_input: bool,
    sampling_rate_hz: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ModbusTcpConfig {
    name: String,
    ip_address: String,
    port: u16,
}

#[derive(Debug, Deserialize)]
pub struct CustomConfig {
    name: String,
    // Add custom configuration fields here
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum TaskConfig {
    Fmu(FmuConfig),
    Gpio(GpioConfig),
    Serial(SerialConfig),
    Udp(UdpConfig),
    Analog(AnalogConfig),
    ModbusTcp(ModbusTcpConfig),
    Custom(CustomConfig),
    // Add other task types here
}

#[derive(Debug, Deserialize)]
pub struct DependencyConfig {
    from: String,
    to: String,
    #[serde(rename = "type")]
    dep_type: String, // "direct" or "memory_block"
    data_flow: String,
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    pub log_file: String,
    pub log_interval_millis: Option<u64>,
    pub logged_outputs: HashMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct SimulationConfig {
    #[serde(default)]
    tasks: Vec<TaskConfig>,
    #[serde(default)]
    dependencies: Vec<DependencyConfig>,
    time_multiplier: Option<f64>,
    pub logging: Option<LoggingConfig>,
}

// Task creator functions
fn create_fmu_task(config: TaskConfig) -> Result<Box<dyn SimulationTask>, FrameworkError> {
    if let TaskConfig::Fmu(cfg) = config {
        Ok(Box::new(FmuTask::new(cfg.name, cfg.path)?))
    } else {
        Err(FrameworkError::ConfigurationError("Invalid config for FMU task".to_string()))
    }
}

fn create_gpio_task(config: TaskConfig) -> Result<Box<dyn SimulationTask>, FrameworkError> {
    if let TaskConfig::Gpio(cfg) = config {
        Ok(Box::new(GpioTask::new(cfg.name, cfg.pins)))
    } else {
        Err(FrameworkError::ConfigurationError("Invalid config for GPIO task".to_string()))
    }
}

fn create_serial_task(config: TaskConfig) -> Result<Box<dyn SimulationTask>, FrameworkError> {
    if let TaskConfig::Serial(cfg) = config {
        Ok(Box::new(SerialTask::new(cfg.name, cfg.port, cfg.baud_rate)))
    } else {
        Err(FrameworkError::ConfigurationError("Invalid config for Serial task".to_string()))
    }
}

fn create_udp_task(config: TaskConfig) -> Result<Box<dyn SimulationTask>, FrameworkError> {
    if let TaskConfig::Udp(cfg) = config {
        Ok(Box::new(UdpTask::new(cfg.name, cfg.local_addr, cfg.remote_addr)))
    } else {
        Err(FrameworkError::ConfigurationError("Invalid config for UDP task".to_string()))
    }
}

fn create_analog_task(config: TaskConfig) -> Result<Box<dyn SimulationTask>, FrameworkError> {
    if let TaskConfig::Analog(cfg) = config {
        Ok(Box::new(AnalogTask::new(cfg.name, cfg.channels, cfg.is_input, cfg.sampling_rate_hz)))
    } else {
        Err(FrameworkError::ConfigurationError("Invalid config for Analog task".to_string()))
    }
}

fn create_modbus_tcp_task(config: TaskConfig) -> Result<Box<dyn SimulationTask>, FrameworkError> {
    if let TaskConfig::ModbusTcp(cfg) = config {
        Ok(Box::new(ModbusTcpTask::new(cfg.name, cfg.ip_address, cfg.port)))
    } else {
        Err(FrameworkError::ConfigurationError("Invalid config for Modbus TCP task".to_string()))
    }
}

fn create_custom_task(config: TaskConfig) -> Result<Box<dyn SimulationTask>, FrameworkError> {
    if let TaskConfig::Custom(cfg) = config {
        Ok(Box::new(CustomTask::new(cfg.name)))
    } else {
        Err(FrameworkError::ConfigurationError("Invalid config for Custom task".to_string()))
    }
}

pub struct Logger {
    writer: Writer<File>,
    headers_written: bool,
    logged_output_names: Vec<String>,
}

impl Logger {
    pub fn new(file_path: &str, logged_output_names: Vec<String>) -> Result<Self, FrameworkError> {
        let file = File::create(file_path).map_err(FrameworkError::IoError)?;
        let writer = Writer::from_writer(file);
        Ok(Logger { writer, headers_written: false, logged_output_names })
    }

    pub fn write_headers(&mut self) -> Result<(), FrameworkError> {
        let mut headers = vec!["Time".to_string()];
        headers.extend(self.logged_output_names.iter().cloned());
        self.writer.write_record(headers).map_err(FrameworkError::CsvError)?;
        self.headers_written = true;
        Ok(())
    }

    pub fn write_record(&mut self, current_time_secs: f64, task_outputs: &HashMap<String, HashMap<String, f64>>) -> Result<(), FrameworkError> {
        if !self.headers_written {
            eprintln!("Warning: Attempted to write data before headers.");
        }
        let mut record = vec![format!("{}", current_time_secs)];
        for output_name in &self.logged_output_names {
            let parts: Vec<&str> = output_name.split(".").collect();
            if parts.len() == 2 {
                let task_name = parts[0];
                let output_var_name = parts[1];
                if let Some(task_output_map) = task_outputs.get(task_name) {
                    if let Some(value) = task_output_map.get(output_var_name) {
                        record.push(format!("{}", value));
                    } else {
                        record.push("".to_string()); // Placeholder for missing value
                    }
                } else {
                    record.push("".to_string()); // Placeholder for missing task
                }
            } else {
                record.push("".to_string()); // Invalid format
            }
        }
        self.writer.write_record(record).map_err(FrameworkError::CsvError)?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), FrameworkError> {
        self.writer.flush().map_err(FrameworkError::IoError)?;
        Ok(())
    }
}

pub async fn send_control_command(command: SimulationStatus) -> Result<(), FrameworkError> {
    let client = reqwest::Client::new();
    let url = match command {
        SimulationStatus::Running => "http://127.0.0.1:3030/control?cmd=start",
        SimulationStatus::Paused => "http://127.0.0.1:3030/control?cmd=pause",
        SimulationStatus::Stopped => "http://127.0.0.1:3030/control?cmd=stop",
    };

    client.get(url).send().await
        .map_err(|e| FrameworkError::WebServerError(e.to_string()))?;

    Ok(())
}

pub async fn start_web_server(sim_graph_arc: Arc<RwLock<SimulationGraph>>, mut shutdown_rx: watch::Receiver<SimulationStatus>) -> Result<(watch::Sender<SimulationStatus>, Arc<RwLock<SimulationData>>, tokio::task::JoinHandle<Result<(), FrameworkError>>, watch::Sender<SimulationStatus>), FrameworkError> {
    println!("start_web_server: Initializing...");
    let (status_tx, _status_rx) = watch::channel(SimulationStatus::Stopped);
    let simulation_data = Arc::new(RwLock::new(SimulationData {
        current_time_secs: 0.0,
        task_outputs: HashMap::new(),
        task_execution_times_micros: HashMap::new(),
    }));

    let data_filter_for_web = Arc::clone(&simulation_data);
    let data_filter = warp::any().map(move || data_filter_for_web.clone());

    let status_tx_for_routes = status_tx.clone();

    let _sim_graph_for_web_data = Arc::clone(&sim_graph_arc);
    let sim_graph_for_web_graph = Arc::clone(&sim_graph_arc);
    let sim_graph_for_web_params = Arc::clone(&sim_graph_arc);
    let sim_graph_for_web_set_params = Arc::clone(&sim_graph_arc);

    let combined_routes = warp::path!("hello")
        .map(|| "Hello, world!")
        .or(warp::path("control").and(warp::query::<HashMap<String, String>>()).map(move |params: HashMap<String, String>| {
            if let Some(cmd) = params.get("cmd") {
                match cmd.as_str() {
                    "start" => {
                        println!("Web server: Received start command.");
                        let _ = status_tx_for_routes.send(SimulationStatus::Running);
                        "Simulation started."
                    },
                    "pause" => {
                        println!("Web server: Received pause command.");
                        let _ = status_tx_for_routes.send(SimulationStatus::Paused);
                        "Simulation paused."
                    },
                    "resume" => {
                        println!("Web server: Received resume command.");
                        let _ = status_tx_for_routes.send(SimulationStatus::Running);
                        "Simulation resumed."
                    },
                    "stop" => {
                        println!("Web server: Received stop command.");
                        let _ = status_tx_for_routes.send(SimulationStatus::Stopped);
                        "Simulation stopped."
                    },
                    _ => "Unknown command.",
                }
            } else {
                "No command specified."
            }
        }))
        .or(warp::path!("data").and(data_filter.clone()).and_then(move |data: Arc<RwLock<SimulationData>>| async move {
            println!("Web server: Data endpoint hit.");
            let data = data.read().await;
            Ok::<_, warp::Rejection>(warp::reply::json(&*data))
        }))
        .or(warp::path!("graph").and(warp::any()).and_then(move || {
            println!("Web server: Graph endpoint hit.");
            let sim_graph_for_web = Arc::clone(&sim_graph_for_web_graph);
            async move {
                let sim_graph_guard = sim_graph_for_web.read().await;
                let mut tasks_info = Vec::new();
                for node_index in sim_graph_guard.graph.node_indices() {
                    let task_arc = sim_graph_guard.graph[node_index].clone();
                    let task = task_arc.read().await;
                    tasks_info.push(WebTaskInfo {
                        name: task.get_name(),
                        inputs: task.get_inputs(),
                        outputs: task.get_outputs(),
                    });
                }

                let mut dependencies_info = Vec::new();
                for edge_index in sim_graph_guard.graph.edge_indices() {
                    let (from_node, to_node) = sim_graph_guard.graph.edge_endpoints(edge_index).unwrap();
                    let from_task_name = sim_graph_guard.graph[from_node].read().await.get_name();
                    let to_task_name = sim_graph_guard.graph[to_node].read().await.get_name();
                    let dep_type = match &sim_graph_guard.graph[edge_index] {
                        DependencyType::Direct(data_flow) => WebDependencyInfo {
                            from_task: from_task_name,
                            to_task: to_task_name,
                            data_flow: data_flow.clone(),
                            dep_type: "direct".to_string(),
                        },
                        DependencyType::MemoryBlock(data_flow) => WebDependencyInfo {
                            from_task: from_task_name,
                            to_task: to_task_name,
                            data_flow: data_flow.clone(),
                            dep_type: "memory_block".to_string(),
                        },
                    };
                    dependencies_info.push(dep_type);
                }

                let graph_info = WebGraphInfo {
                    tasks: tasks_info,
                    dependencies: dependencies_info,
                };
                Ok::<_, warp::Rejection>(warp::reply::json(&graph_info))
            }
        }))
        .or(warp::path!("parameters").and(warp::any()).and_then(move || {
            println!("Web server: Parameters endpoint hit.");
            let sim_graph_for_web = Arc::clone(&sim_graph_for_web_params);
            async move {
                let sim_graph_guard = sim_graph_for_web.read().await;
                let mut all_parameters: HashMap<String, HashMap<String, Parameter>> = HashMap::new();
                for node_index in sim_graph_guard.graph.node_indices() {
                    let task_arc = sim_graph_guard.graph[node_index].clone();
                    let task = task_arc.read().await;
                    let parameters = task.get_parameters();
                    if !parameters.is_empty() {
                        all_parameters.insert(task.get_name(), parameters);
                    }
                }
                Ok::<_, warp::Rejection>(warp::reply::json(&all_parameters))
            }
        }))
        .or(warp::path!("parameters" / "set").and(warp::post()).and(warp::body::json()).and(warp::any()).and_then(move |req: SetParameterRequest| {
            println!("Web server: Set parameters endpoint hit.");
            let sim_graph_for_web = Arc::clone(&sim_graph_for_web_set_params);
            async move {
                let sim_graph_guard = sim_graph_for_web.write().await;
                for node_index in sim_graph_guard.graph.node_indices() {
                    let task_arc = sim_graph_guard.graph[node_index].clone();
                    let mut task = task_arc.write().await;
                    if task.get_name() == req.task_name {
                        match task.set_parameter(&req.param_name, req.param_value) {
                            Ok(_) => return Ok::<_, warp::Rejection>(warp::reply::json(&"Parameter set successfully.")),
                            Err(e) => return Err(warp::reject::custom(e)),
                        }
                    }
                }
                Err(warp::reject::custom(FrameworkError::ConfigurationError(format!("Task '{}' not found.", req.task_name))))
            }
        }));

    let server_handle = tokio::spawn(async move {
        let addr = ([127, 0, 0, 1], 3030);
        println!("Web server bound to address: {:?}", addr);
        let (_, server) = warp::serve(combined_routes).bind_with_graceful_shutdown(addr, async move {
            shutdown_rx.changed().await.ok();
            println!("Web server shutting down.");
        });
        server.await;
        println!("Web server task finished gracefully.");
        Ok::<(), FrameworkError>(())
    });

    println!("Web server running on http://127.0.0.1:3030/hello");
    println!("start_web_server: Initialization complete.");

    Ok((status_tx.clone(), simulation_data, server_handle, status_tx))
}

pub async fn run_framework(
    simulation_duration_secs_cli: u64,
    time_step_millis_cli: u64,
    config_file_path: Option<PathBuf>,
    mut status_rx: watch::Receiver<SimulationStatus>,
    simulation_data: Arc<RwLock<SimulationData>>,
    sim_graph_arc: Arc<RwLock<SimulationGraph>>,
) -> Result<(), FrameworkError> {
    println!("run_framework: Starting...");

    let config: SimulationConfig = {
        let path = config_file_path.ok_or(FrameworkError::ConfigurationError("Configuration file path not provided.".to_string()))?;
        println!("run_framework: Reading config from {}", path.display());
        let config_content = fs::read_to_string(&path).await?;
        
        let deserialized_config: SimulationConfig = toml::from_str(&config_content)?;
        deserialized_config
    };

    let simulation_duration = Duration::from_secs(simulation_duration_secs_cli);
    let mut time_step = Duration::from_millis(time_step_millis_cli);

    // Apply time multiplier from config if present
    if let Some(multiplier) = config.time_multiplier {
        if multiplier > 0.0 {
            time_step = Duration::from_secs_f64(time_step.as_secs_f64() / multiplier);
            println!("run_framework: Applying time multiplier: {}. Adjusted time step: {:?}", multiplier, time_step);
        }
    }

    let mut interval = time::interval(time_step);
    let mut current_time = Duration::from_secs(0);

    println!("run_framework: Starting simulation loop...");

    let mut sim_graph = SimulationGraph::new();
    let mut task_factory = TaskFactory::new();
    let mut io_task_indices: Vec<NodeIndex> = Vec::new();

    // Register task creators
    task_factory.register_task("Fmu", create_fmu_task);
    task_factory.register_task("Gpio", create_gpio_task);
    task_factory.register_task("Serial", create_serial_task);
    task_factory.register_task("Udp", create_udp_task);
    task_factory.register_task("Analog", create_analog_task);
    task_factory.register_task("ModbusTcp", create_modbus_tcp_task);
    task_factory.register_task("Custom", create_custom_task);

    // Populate sim_graph from config
    for task_config in config.tasks {
        println!("run_framework: Creating task: {:?}", task_config);
        let task = task_factory.create_task(task_config)?;
        let task_name = task.get_name();
        let is_io = task.as_any().is::<GpioTask>() ||
                     task.as_any().is::<SerialTask>() ||
                     task.as_any().is::<UdpTask>() ||
                     task.as_any().is::<AnalogTask>() ||
                     task.as_any().is::<ModbusTcpTask>();
        let task_arc = Arc::new(RwLock::new(task));
        let node_index = sim_graph.add_task(task_name, task_arc.clone());
        if is_io {
            io_task_indices.push(node_index);
        }
    }

    for dep_config in config.dependencies {
        println!("run_framework: Adding dependency: {:?}", dep_config);
        let dep_type = match dep_config.dep_type.as_str() {
            "direct" => DependencyType::Direct(dep_config.data_flow),
            "memory_block" => DependencyType::MemoryBlock(dep_config.data_flow),
            _ => return Err(FrameworkError::ConfigurationError(format!("Unknown dependency type: {}", dep_config.dep_type))),
        };
        sim_graph.add_dependency(&dep_config.from, &dep_config.to, dep_type)?;
    }


    let execution_order = sim_graph.get_execution_order()?;
    println!("run_framework: Execution order determined.");

    *sim_graph_arc.write().await = sim_graph; // Update the shared sim_graph_arc

    // Initialize I/O tasks
    for node_index in &io_task_indices {
        let task_arc = sim_graph_arc.read().await.graph[*node_index].clone();
        let mut task = task_arc.write().await;
        println!("run_framework: Initializing I/O for task: {}", task.get_name());
        if let Some(io_task) = task.as_any_mut().downcast_mut::<GpioTask>() {
            if let Err(e) = io_task.initialize_io().await {
                eprintln!("Error initializing GPIO task {}: {}", io_task.get_name(), e);
                return Err(e);
            }
        } else if let Some(io_task) = task.as_any_mut().downcast_mut::<SerialTask>() {
            if let Err(e) = io_task.initialize_io().await {
                eprintln!("Error initializing Serial task {}: {}", io_task.get_name(), e);
                return Err(e);
            }
        } else if let Some(io_task) = task.as_any_mut().downcast_mut::<UdpTask>() {
            if let Err(e) = io_task.initialize_io().await {
                eprintln!("Error initializing UDP task {}: {}", io_task.get_name(), e);
                return Err(e);
            }
        } else if let Some(io_task) = task.as_any_mut().downcast_mut::<AnalogTask>() {
            if let Err(e) = io_task.initialize_io().await {
                eprintln!("Error initializing Analog task {}: {}", io_task.get_name(), e);
                return Err(e);
            }
        } else if let Some(io_task) = task.as_any_mut().downcast_mut::<ModbusTcpTask>() {
            if let Err(e) = io_task.initialize_io().await {
                eprintln!("Error initializing Modbus TCP task {}: {}", io_task.get_name(), e);
                return Err(e);
            }
        }
    }

    let mut logger = if let Some(logging_config) = &config.logging {
        println!("run_framework: Initializing logger with config: {:?}", logging_config);
        let mut logged_output_names = Vec::new();
        for (task_name, outputs) in &logging_config.logged_outputs {
            for output_name in outputs {
                logged_output_names.push(format!("{}.{}", task_name, output_name));
            }
        }
        Logger::new(&logging_config.log_file, logged_output_names)?
    } else {
        println!("run_framework: Initializing default logger.");
        Logger::new("simulation_log.csv", vec!["FMU1.output_var".to_string()])?
    };

    // Write CSV headers
    logger.write_headers()?;
    println!("run_framework: CSV headers written.");

    // REQ-DETERMINISTIC-EXECUTION: Ensuring deterministic execution is crucial for real-time simulations.
    // This involves careful consideration of:
    // - Floating-point arithmetic: Use fixed-point or ensure consistent FPU settings across platforms.
    // - Random number generation: Use deterministic PRNGs with fixed seeds.
    // - Thread scheduling: Rely on RTOS features (e.g., Linux RT patch) for predictable task execution.
    // - External interactions: Minimize non-deterministic I/O or provide mechanisms to make it deterministic.

    // REQ-ERROR-HANDLING: Robust error handling is critical for real-time systems.
    // The framework aims to provide informative error messages and graceful exits
    // upon critical failures during configuration, graph construction, or I/O operations.
    // Future enhancements may include more sophisticated error recovery mechanisms.

    loop {
        tokio::select! {
            _ = interval.tick() => {
                // Handle tick
                println!("run_framework: Tick received.");
            }
            res = status_rx.changed() => {
                println!("run_framework: Status change detected.");
                if res.is_err() {
                    println!("run_framework: Status sender dropped, exiting loop.");
                    break;
                }
                // Status changed, re-evaluate in next iteration
            }
        }

        let current_status = status_rx.borrow().clone();
        println!("run_framework: Current status: {:?}", current_status);

        match current_status {
            SimulationStatus::Running => {
                current_time += time_step;
                println!("run_framework: Simulation Time: {:?}", current_time);

                // Read all I/O inputs
                for node_index in &io_task_indices {
                    let task_arc = sim_graph_arc.read().await.graph[*node_index].clone();
                    let mut task = task_arc.write().await;
                    println!("run_framework: Reading I/O for task: {}", task.get_name());
                    if let Some(io_task) = task.as_any_mut().downcast_mut::<GpioTask>() {
                        if let Err(e) = io_task.read_io().await {
                            eprintln!("Error reading from GPIO task {}: {}", io_task.get_name(), e);
                        }
                    }
                    if let Some(io_task) = task.as_any_mut().downcast_mut::<SerialTask>() {
                        if let Err(e) = io_task.read_io().await {
                            eprintln!("Error reading from Serial task {}: {}", io_task.get_name(), e);
                        }
                    }
                    if let Some(io_task) = task.as_any_mut().downcast_mut::<UdpTask>() {
                        if let Err(e) = io_task.read_io().await {
                            eprintln!("Error reading from UDP task {}: {}", io_task.get_name(), e);
                        }
                    }
                    if let Some(io_task) = task.as_any_mut().downcast_mut::<AnalogTask>() {
                        if let Err(e) = io_task.read_io().await {
                            eprintln!("Error reading from Analog task {}: {}", io_task.get_name(), e);
                        }
                    }
                    if let Some(io_task) = task.as_any_mut().downcast_mut::<ModbusTcpTask>() {
                        if let Err(e) = io_task.read_io().await {
                            eprintln!("Error reading from Modbus TCP task {}: {}", io_task.get_name(), e);
                        }
                    }
                }

                // Update shared simulation data
                let mut sim_data_guard = simulation_data.write().await;
                sim_data_guard.current_time_secs = current_time.as_secs_f64();
                sim_data_guard.task_outputs.clear();
                sim_data_guard.task_execution_times_micros.clear();

                for node_index in &execution_order {
                    let task_arc = sim_graph_arc.read().await.graph[*node_index].clone();
                    let mut task = task_arc.write().await;

                    let start_time = time::Instant::now();
                    println!("run_framework: Executing task: {}", task.get_name());
                    if let Err(e) = task.execute(current_time).await {
                        eprintln!("Error executing task {}: {}", task.get_name(), e);
                        // Here we could implement different recovery mechanisms:
                        // 1. Skip this task for the current step.
                        // 2. Attempt to re-initialize the task.
                        // 3. Stop the simulation gracefully.
                        // For now, we'll just log and continue, but a more robust solution
                        // would involve a configurable error strategy.
                    }
                    let end_time = time::Instant::now();
                    let elapsed_micros = (end_time - start_time).as_micros() as u64;

                    // Store execution time
                    sim_data_guard.task_execution_times_micros.insert(
                        task.get_name(),
                        elapsed_micros,
                    );

                    // Collect outputs for monitoring
                    let mut task_output_map = HashMap::new();
                    for output_name in task.get_outputs() {
                        if let Some(output_val) = task.get_output_value(&output_name) {
                            task_output_map.insert(output_name, output_val);
                        }
                    }
                    if !task_output_map.is_empty() {
                        sim_data_guard.task_outputs.insert(task.get_name(), task_output_map);
                    }

                    // Log FMU1 output_var
                    if let Err(e) = logger.write_record(current_time.as_secs_f64(), &sim_data_guard.task_outputs) {
                        eprintln!("Failed to write log record: {}", e);
                    }
                }
                drop(sim_data_guard); // Release the write lock

                // Write all I/O outputs
                for node_index in &io_task_indices {
                    let task_arc = sim_graph_arc.read().await.graph[*node_index].clone();
                    let mut task = task_arc.write().await;
                    println!("run_framework: Writing I/O for task: {}", task.get_name());
                    if let Some(io_task) = task.as_any_mut().downcast_mut::<GpioTask>() {
                        if let Err(e) = io_task.write_io().await {
                            eprintln!("Error writing to GPIO task {}: {}", io_task.get_name(), e);
                        }
                    }
                    if let Some(io_task) = task.as_any_mut().downcast_mut::<SerialTask>() {
                        if let Err(e) = io_task.write_io().await {
                            eprintln!("Error writing to Serial task {}: {}", io_task.get_name(), e);
                        }
                    }
                    if let Some(io_task) = task.as_any_mut().downcast_mut::<UdpTask>() {
                        if let Err(e) = io_task.write_io().await {
                            eprintln!("Error writing to UDP task {}: {}", io_task.get_name(), e);
                        }
                    }
                    if let Some(io_task) = task.as_any_mut().downcast_mut::<AnalogTask>() {
                        if let Err(e) = io_task.write_io().await {
                            eprintln!("Error writing to Analog task {}: {}", io_task.get_name(), e);
                        }
                    }
                    if let Some(io_task) = task.as_any_mut().downcast_mut::<ModbusTcpTask>() {
                        if let Err(e) = io_task.write_io().await {
                            eprintln!("Error writing to Modbus TCP task {}: {}", io_task.get_name(), e);
                        }
                    }
                }

                if current_time >= simulation_duration {
                    println!("run_framework: Simulation finished.");
                    break;
                }
            },
            SimulationStatus::Paused => {
                println!("run_framework: Simulation Paused at Time: {:?}", current_time);
                // Do nothing, just wait for status change
            },
            SimulationStatus::Stopped => {
                println!("run_framework: Simulation Stopped.");
                break;
            },
        }
    }

    logger.flush()?;
    println!("run_framework: Logger flushed.");

    Ok(())
}
