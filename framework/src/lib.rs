use std::path::PathBuf;
use tokio::fs;
use tokio::time::{self, Duration};
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

/// Defines the interface for any simulation task within the framework.
/// Custom components should implement this trait to be integrated into the simulation graph.
#[async_trait]
pub trait SimulationTask: Send + Sync + 'static {
    /// Executes the task for the current simulation time step.
    /// Implementations should perform their core logic here, such as updating internal states,
    /// processing inputs, and producing outputs.
    async fn execute(&mut self, current_time: Duration);
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
}

impl FmuTask {
    pub fn new(name: String) -> Self {
        FmuTask { 
            name,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
        }
    }
}

#[async_trait]
impl SimulationTask for FmuTask {
    async fn execute(&mut self, current_time: Duration) {
        println!("  Executing FMU Task {}: {:?}", self.name, current_time);
        // In a real FMU, this would involve calling the FMU's do_step method
        // and updating internal states based on inputs and producing outputs.
        // For now, let's just simulate some output change
        if let Some(output_val) = self.outputs.get_mut("output_var") {
            *output_val = current_time.as_secs_f64();
        }
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
}

impl GpioTask {
    pub fn new(name: String, pins: Vec<u8>) -> Self {
        GpioTask { 
            name,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            pins,
        }
    }
}

#[async_trait]
impl SimulationTask for GpioTask {
    async fn execute(&mut self, _current_time: Duration) {
        println!("  Executing GPIO Task {}", self.name);
        // Call trait methods explicitly
        let _ = <Self as IoTask>::read_io(self).await;
        // Process inputs if any
        let _ = <Self as IoTask>::write_io(self).await;
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
}

impl SerialTask {
    pub fn new(name: String, port: String, baud_rate: u32) -> Self {
        SerialTask { 
            name,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            port,
            baud_rate,
        }
    }
}

#[async_trait]
impl SimulationTask for SerialTask {
    async fn execute(&mut self, _current_time: Duration) {
        println!("  Executing Serial Task {}", self.name);
        let _ = <Self as IoTask>::read_io(self).await;
        // Process inputs if any
        let _ = <Self as IoTask>::write_io(self).await;
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

use tokio::net::UdpSocket;

// Placeholder for UDP Communication Task
pub struct UdpTask {
    name: String,
    inputs: HashMap<String, f64>,
    outputs: HashMap<String, f64>,
    socket: Option<UdpSocket>,
    local_addr: String,
    remote_addr: String,
}

impl UdpTask {
    pub fn new(name: String, local_addr: String, remote_addr: String) -> Self {
        UdpTask { 
            name,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            socket: None,
            local_addr,
            remote_addr,
        }
    }
}

#[async_trait]
impl SimulationTask for UdpTask {
    async fn execute(&mut self, _current_time: Duration) {
        println!("  Executing UDP Task {}", self.name);
        let _ = <Self as IoTask>::read_io(self).await;
        // Process inputs if any
        let _ = <Self as IoTask>::write_io(self).await;
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
                    // For now, just print the error. Will handle properly later.
                    eprintln!("UDP read error: {}", e);
                }
            }
        }
        Ok(())
    }

    async fn write_io(&mut self) -> Result<(), FrameworkError> {
        if let Some(socket) = &self.socket {
            if let Some(value) = self.get_input_value("udp_input") {
                let data = value.to_be_bytes();
                match socket.send(&data).await {
                    Ok(_) => {
                        println!("    UDP Task {} sent value: {}", self.name, value);
                    },
                    Err(e) => {
                        // For now, just print the error. Will handle properly later.
                        eprintln!("UDP write error: {}", e);
                    }
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
}

impl AnalogTask {
    pub fn new(name: String, channels: Vec<u8>, is_input: bool, sampling_rate_hz: Option<u32>) -> Self {
        AnalogTask {
            name,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            channels,
            is_input,
            sampling_rate_hz,
        }
    }
}

#[async_trait]
impl SimulationTask for AnalogTask {
    async fn execute(&mut self, _current_time: Duration) {
        println!("  Executing Analog Task {}", self.name);
        if self.is_input {
            let _ = <Self as IoTask>::read_io(self).await;
        } else {
            let _ = <Self as IoTask>::write_io(self).await;
        }
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
}

#[async_trait]
impl IoTask for AnalogTask {
    async fn initialize_io(&mut self) -> Result<(), FrameworkError> {
        print!("    Analog Task {} initialized for channels: {:?}. Is input: {}.", self.name, self.channels, self.is_input);
        if let Some(rate) = self.sampling_rate_hz {
            println!(" Sampling Rate: {} Hz.", rate);
        } else {
            println!(" No sampling rate specified.");
        }
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
}

impl ModbusTcpTask {
    pub fn new(name: String, ip_address: String, port: u16) -> Self {
        ModbusTcpTask {
            name,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            ip_address,
            port,
        }
    }
}

#[async_trait]
impl SimulationTask for ModbusTcpTask {
    async fn execute(&mut self, _current_time: Duration) {
        println!("  Executing Modbus TCP Task {}", self.name);
        let _ = <Self as IoTask>::read_io(self).await;
        let _ = <Self as IoTask>::write_io(self).await;
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
    // Add custom fields here
}

impl CustomTask {
    pub fn new(name: String) -> Self {
        CustomTask {
            name,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            // Initialize custom fields
        }
    }
}

#[async_trait]
impl SimulationTask for CustomTask {
    async fn execute(&mut self, current_time: Duration) {
        println!("  Executing Custom Task {}: {:?}", self.name, current_time);
        // Implement custom logic here
        // For example, update outputs based on inputs and current_time
        if let Some(output_val) = self.outputs.get_mut("custom_output") {
            *output_val = current_time.as_secs_f64() * 2.0;
        }
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
}

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
    tasks: Vec<TaskConfig>,
    dependencies: Vec<DependencyConfig>,
    time_multiplier: Option<f64>,
    pub logging: Option<LoggingConfig>,
}

// Task creator functions
fn create_fmu_task(config: TaskConfig) -> Result<Box<dyn SimulationTask>, FrameworkError> {
    if let TaskConfig::Fmu(cfg) = config {
        Ok(Box::new(FmuTask::new(cfg.name)))
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

pub async fn start_web_server() -> Result<(watch::Sender<SimulationStatus>, Arc<RwLock<SimulationData>>, tokio::task::JoinHandle<Result<(), FrameworkError>>), FrameworkError> {
    let (status_tx, _status_rx) = watch::channel(SimulationStatus::Stopped);
    let simulation_data = Arc::new(RwLock::new(SimulationData {
        current_time_secs: 0.0,
        task_outputs: HashMap::new(),
        task_execution_times_micros: HashMap::new(),
    }));

    let data_filter_for_web = Arc::clone(&simulation_data);
    let data_filter = warp::any().map(move || data_filter_for_web.clone());

    let status_tx_for_routes = status_tx.clone();

    let combined_routes = warp::path!("hello")
        .map(|| "Hello, world!")
        .or(warp::path("control").and(warp::query::<HashMap<String, String>>()).map(move |params: HashMap<String, String>| {
            if let Some(cmd) = params.get("cmd") {
                match cmd.as_str() {
                    "start" => {
                        let _ = status_tx_for_routes.send(SimulationStatus::Running);
                        "Simulation started."
                    },
                    "pause" => {
                        let _ = status_tx_for_routes.send(SimulationStatus::Paused);
                        "Simulation paused."
                    },
                    "resume" => {
                        let _ = status_tx_for_routes.send(SimulationStatus::Running);
                        "Simulation resumed."
                    },
                    "stop" => {
                        let _ = status_tx_for_routes.send(SimulationStatus::Stopped);
                        "Simulation stopped."
                    },
                    _ => "Unknown command.",
                }
            } else {
                "No command specified."
            }
        }))
        .or(warp::path!("data").and(data_filter).and_then(|data: Arc<RwLock<SimulationData>>| async move {
            let data = data.read().await;
            Ok::<_, warp::Rejection>(warp::reply::json(&*data))
        }));

    let server_handle = tokio::spawn(async move {
        let (addr, server) = warp::serve(combined_routes).bind(([127, 0, 0, 1], 3030)).await;
        println!("Web server bound to address: {:?}", addr);
        server.await;
        Ok::<(), FrameworkError>(()) // Explicitly return Ok on graceful shutdown
    });

    println!("Web server running on http://127.0.0.1:3030/hello");

    Ok((status_tx, simulation_data, server_handle))
}

pub async fn run_framework(
    simulation_duration_secs_cli: u64,
    time_step_millis_cli: u64,
    config_file_path: Option<PathBuf>,
    mut status_rx: watch::Receiver<SimulationStatus>,
    simulation_data: Arc<RwLock<SimulationData>>,
) -> Result<(), FrameworkError> {
    println!("Framework is running...");

    let config: SimulationConfig = {
        let path = config_file_path.ok_or(FrameworkError::ConfigurationError("Configuration file path not provided.".to_string()))?;
        let config_content = fs::read_to_string(&path).await.map_err(|e| FrameworkError::IoError(e))?;
        toml::from_str(&config_content).map_err(|e| FrameworkError::TomlError(e))?
    };

    println!("Loaded Configuration: {:#?}", config);

    let simulation_duration = Duration::from_secs(simulation_duration_secs_cli);
    let mut time_step = Duration::from_millis(time_step_millis_cli);

    // Apply time multiplier from config if present
    if let Some(multiplier) = config.time_multiplier {
        time_step = Duration::from_secs_f64(time_step.as_secs_f64() / multiplier);
        println!("Applying time multiplier: {}. Adjusted time step: {:?}", multiplier, time_step);
    }

    let mut interval = time::interval(time_step);
    let mut current_time = Duration::from_secs(0);

    println!("Starting simulation loop...");

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
        let task = task_factory.create_task(task_config)?;
        let task_arc = Arc::new(RwLock::new(task));
        let node_index = sim_graph.add_task(task_arc.read().await.get_name(), task_arc.clone());
        // Check if the task is an IoTask and store its index
        if task_arc.read().await.as_any().is::<GpioTask>() ||
           task_arc.read().await.as_any().is::<SerialTask>() ||
           task_arc.read().await.as_any().is::<UdpTask>() ||
           task_arc.read().await.as_any().is::<AnalogTask>() ||
           task_arc.read().await.as_any().is::<ModbusTcpTask>() {
            io_task_indices.push(node_index);
        }
    }

    for dep_config in config.dependencies {
        let dep_type = match dep_config.dep_type.as_str() {
            "direct" => DependencyType::Direct(dep_config.data_flow),
            "memory_block" => DependencyType::MemoryBlock(dep_config.data_flow),
            _ => return Err(FrameworkError::ConfigurationError(format!("Unknown dependency type: {}", dep_config.dep_type))),
        };
        sim_graph.add_dependency(&dep_config.from, &dep_config.to, dep_type)?;
    }

    let execution_order = sim_graph.get_execution_order()?;

    // Initialize I/O tasks
    for node_index in &io_task_indices {
        let task_arc = sim_graph.graph[*node_index].clone();
        let mut task = task_arc.write().await;
        if let Some(io_task) = task.as_any_mut().downcast_mut::<GpioTask>() {
            io_task.initialize_io().await?;
        } else if let Some(io_task) = task.as_any_mut().downcast_mut::<SerialTask>() {
            io_task.initialize_io().await?;
        } else if let Some(io_task) = task.as_any_mut().downcast_mut::<UdpTask>() {
            io_task.initialize_io().await?;
        } else if let Some(io_task) = task.as_any_mut().downcast_mut::<AnalogTask>() {
            io_task.initialize_io().await?;
        } else if let Some(io_task) = task.as_any_mut().downcast_mut::<ModbusTcpTask>() {
            io_task.initialize_io().await?;
        }
    }

    let mut logger = if let Some(logging_config) = &config.logging {
        let mut logged_output_names = Vec::new();
        for (task_name, outputs) in &logging_config.logged_outputs {
            for output_name in outputs {
                logged_output_names.push(format!("{}.{}", task_name, output_name));
            }
        }
        Logger::new(&logging_config.log_file, logged_output_names)?
    } else {
        Logger::new("simulation_log.csv", vec!["FMU1.output_var".to_string()])?
    };

    // Write CSV headers
    logger.write_headers()?;

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
            }
            res = status_rx.changed() => {
                if res.is_err() {
                    // Sender dropped, exit loop
                    break;
                }
                // Status changed, re-evaluate in next iteration
            }
        }

        let current_status = status_rx.borrow().clone();

        match current_status {
            SimulationStatus::Running => {
                current_time += time_step;
                println!("Simulation Time: {:?}", current_time);

                // Read all I/O inputs
                for node_index in &io_task_indices {
                    let task_arc = sim_graph.graph[*node_index].clone();
                    let mut task = task_arc.write().await;
                    if let Some(io_task) = task.as_any_mut().downcast_mut::<GpioTask>() {
                        io_task.read_io().await?;
                    } else if let Some(io_task) = task.as_any_mut().downcast_mut::<SerialTask>() {
                        io_task.read_io().await?;
                    } else if let Some(io_task) = task.as_any_mut().downcast_mut::<UdpTask>() {
                        io_task.read_io().await?;
                    } else if let Some(io_task) = task.as_any_mut().downcast_mut::<AnalogTask>() {
                        io_task.read_io().await?;
                    } else if let Some(io_task) = task.as_any_mut().downcast_mut::<ModbusTcpTask>() {
                        io_task.read_io().await?;
                    }
                }

                // Update shared simulation data
                let mut sim_data_guard = simulation_data.write().await;
                sim_data_guard.current_time_secs = current_time.as_secs_f64();
                sim_data_guard.task_outputs.clear();
                sim_data_guard.task_execution_times_micros.clear();

                for node_index in &execution_order {
                    let task_arc = sim_graph.graph[*node_index].clone();
                    let mut task = task_arc.write().await;

                    let start_time = time::Instant::now();
                    task.execute(current_time).await;
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
                    let task_arc = sim_graph.graph[*node_index].clone();
                    let mut task = task_arc.write().await;
                    if let Some(io_task) = task.as_any_mut().downcast_mut::<GpioTask>() {
                        io_task.write_io().await?;
                    }
                    if let Some(io_task) = task.as_any_mut().downcast_mut::<SerialTask>() {
                        io_task.write_io().await?;
                    }
                    if let Some(io_task) = task.as_any_mut().downcast_mut::<UdpTask>() {
                        io_task.write_io().await?;
                    }
                    if let Some(io_task) = task.as_any_mut().downcast_mut::<AnalogTask>() {
                        io_task.write_io().await?;
                    }
                    if let Some(io_task) = task.as_any_mut().downcast_mut::<ModbusTcpTask>() {
                        io_task.write_io().await?;
                    }
                }

                if current_time >= simulation_duration {
                    println!("Simulation finished.");
                    break;
                }
            },
            SimulationStatus::Paused => {
                println!("Simulation Paused at Time: {:?}", current_time);
                // Do nothing, just wait for status change
            },
            SimulationStatus::Stopped => {
                println!("Simulation Stopped.");
                break;
            },
        }
    }

    logger.flush()?;

    Ok(())
}
