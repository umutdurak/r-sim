

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
    WebServerError(#[from] warp::Error),
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

// Placeholder for a concrete FMU task (will be replaced by actual FMU integration)
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
}

impl AnalogTask {
    pub fn new(name: String, channels: Vec<u8>, is_input: bool) -> Self {
        AnalogTask {
            name,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            channels,
            is_input,
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
}

#[async_trait]
impl IoTask for AnalogTask {
    async fn initialize_io(&mut self) -> Result<(), FrameworkError> {
        println!("    Analog Task {} initialized for channels: {:?}. Is input: {}", self.name, self.channels, self.is_input);
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
}

// Represents the type of dependency between tasks
pub enum DependencyType {
    Direct(String), // Direct data flow, e.g., "output_to_input"
    MemoryBlock(String), // Introduces a one-step delay, breaking causality
}

// Represents the simulation graph with tasks and their dependencies
pub struct SimulationGraph {
    graph: DiGraph<Box<dyn SimulationTask>, DependencyType>,
    task_indices: HashMap<String, NodeIndex>,
}

impl SimulationGraph {
    pub fn new() -> Self {
        SimulationGraph {
            graph: DiGraph::new(),
            task_indices: HashMap::new(),
        }
    }

    pub fn add_task(&mut self, name: String, task: Box<dyn SimulationTask>) {
        let index = self.graph.add_node(task);
        self.task_indices.insert(name, index);
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
pub struct SimulationConfig {
    tasks: Vec<TaskConfig>,
    dependencies: Vec<DependencyConfig>,
    time_multiplier: Option<f64>,
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
        Ok(Box::new(AnalogTask::new(cfg.name, cfg.channels, cfg.is_input)))
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
}

impl Logger {
    pub fn new(file_path: &str) -> Result<Self, FrameworkError> {
        let file = File::create(file_path).map_err(FrameworkError::IoError)?;
        let writer = Writer::from_writer(file);
        Ok(Logger { writer, headers_written: false })
    }

    pub fn write_headers(&mut self, headers: &[&str]) -> Result<(), FrameworkError> {
        self.writer.write_record(headers).map_err(FrameworkError::CsvError)?;
        self.headers_written = true;
        Ok(())
    }

    pub fn write_record(&mut self, record: &[&str]) -> Result<(), FrameworkError> {
        if !self.headers_written {
            eprintln!("Warning: Attempted to write data before headers.");
        }
        self.writer.write_record(record).map_err(FrameworkError::CsvError)?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), FrameworkError> {
        self.writer.flush().map_err(FrameworkError::IoError)?;
        Ok(())
    }
}


pub async fn run_framework(
    simulation_duration_secs_cli: u64,
    time_step_millis_cli: u64,
    config_file_path: Option<PathBuf>,
) -> Result<(), FrameworkError> {
    println!("Framework is running...");

    let config: SimulationConfig = if let Some(path) = config_file_path {
        let config_content = fs::read_to_string(&path).await.map_err(|e| FrameworkError::IoError(e))?;
        toml::from_str(&config_content).map_err(|e| FrameworkError::TomlError(e))?
    } else {
        let config_str = r#"
        [[tasks]]
        type = "Fmu"
        name = "FMU1"
        path = "./fmus/fmu1.fmu"

        [[tasks]]
        type = "Fmu"
        name = "FMU2"
        path = "./fmus/fmu2.fmu"

        [[tasks]]
        type = "Gpio"
        name = "GPIO_In"
        pins = [1, 2, 3]

        [[tasks]]
        type = "Serial"
        name = "Serial_Out"
        port = "/dev/ttyUSB0"
        baud_rate = 115200

        [[tasks]]
        type = "Udp"
        name = "UDP_Comm"
        local_addr = "127.0.0.1:8080"
        remote_addr = "127.0.0.1:8081"

        [[dependencies]]
        from = "FMU1"
        to = "FMU2"
        type = "direct"
        data_flow = "output_to_input"

        [[dependencies]]
        from = "FMU2"
        to = "FMU1"
        type = "memory_block"
        data_flow = "feedback_signal"

        [[dependencies]]
        from = "GPIO_In"
        to = "FMU1"
        type = "direct"
        data_flow = "gpio_data"

        [[dependencies]]
        from = "FMU2"
        to = "Serial_Out"
        type = "direct"
        data_flow = "serial_data"

        [[dependencies]]
        from = "UDP_Comm"
        to = "FMU1"
        type = "direct"
        data_flow = "udp_input"
        "#;
        toml::from_str(config_str).map_err(|e| FrameworkError::TomlError(e))?
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
        sim_graph.add_task(task.as_any().downcast_ref::<FmuTask>().unwrap().name.clone(), task);
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
    for node_index in sim_graph.graph.node_indices() {
        let task = &mut sim_graph.graph[node_index];
        if let Some(io_task) = task.as_any_mut().downcast_mut::<UdpTask>() {
            io_task.initialize_io().await?;
        } else if let Some(io_task) = task.as_any_mut().downcast_mut::<GpioTask>() {
            io_task.initialize_io().await?;
        } else if let Some(io_task) = task.as_any_mut().downcast_mut::<SerialTask>() {
            io_task.initialize_io().await?;
        } else if let Some(io_task) = task.as_any_mut().downcast_mut::<AnalogTask>() {
            io_task.initialize_io().await?;
        } else if let Some(io_task) = task.as_any_mut().downcast_mut::<ModbusTcpTask>() {
            io_task.initialize_io().await?;
        }
    }

    let mut logger = Logger::new("simulation_log.csv")?;

    // Write CSV headers
    if let Err(e) = logger.write_headers(&["Time", "FMU1_output_var"]) {
        eprintln!("Failed to write CSV headers: {}", e);
        return Err(e);
    }

    // Web server for monitoring
    let _routes_hello = warp::path!("hello")
        .map(|| "Hello, world!");

    let (status_tx, mut status_rx) = watch::channel(SimulationStatus::Stopped);
    let simulation_data = Arc::new(RwLock::new(SimulationData {
        current_time_secs: 0.0,
        task_outputs: HashMap::new(),
    }));

    let data_filter_for_web = Arc::clone(&simulation_data);
    let data_filter = warp::any().map(move || data_filter_for_web.clone());

    let simulation_data_for_loop = Arc::clone(&simulation_data);

    let combined_routes = _routes_hello
        .or(warp::path("control").and(warp::query::<HashMap<String, String>>()).map(move |params: HashMap<String, String>| {
            if let Some(cmd) = params.get("cmd") {
                match cmd.as_str() {
                    "start" => {
                        let _ = status_tx.send(SimulationStatus::Running);
                        "Simulation started."
                    },
                    "pause" => {
                        let _ = status_tx.send(SimulationStatus::Paused);
                        "Simulation paused."
                    },
                    "resume" => {
                        let _ = status_tx.send(SimulationStatus::Running);
                        "Simulation resumed."
                    },
                    "stop" => {
                        let _ = status_tx.send(SimulationStatus::Stopped);
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

    let (tx, rx) = tokio::sync::oneshot::channel();
    let server_handle = tokio::spawn(async move {
        let (_addr, server) = warp::serve(combined_routes).bind_with_graceful_shutdown(([127, 0, 0, 1], 3030), async {
            rx.await.ok();
        });
        server.await;
    });

    println!("Web server running on http://127.0.0.1:3030/hello");

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

                // Update shared simulation data
                let mut sim_data_guard = simulation_data_for_loop.write().await;
                sim_data_guard.current_time_secs = current_time.as_secs_f64();
                sim_data_guard.task_outputs.clear();

                for node_index in &execution_order {
                    let task = &mut sim_graph.graph[*node_index];
                    task.execute(current_time).await;

                    // Collect outputs for monitoring
                    let mut task_output_map = HashMap::new();
                    for output_name in task.get_outputs() {
                        if let Some(output_val) = task.get_output_value(&output_name) {
                            task_output_map.insert(output_name, output_val);
                        }
                    }
                    if !task_output_map.is_empty() {
                        sim_data_guard.task_outputs.insert(task.as_any().downcast_ref::<FmuTask>().unwrap().name.clone(), task_output_map);
                    }

                    // Log FMU1 output_var
                    if let Some(fmu_task) = task.as_any_mut().downcast_mut::<FmuTask>() {
                        if fmu_task.name == "FMU1" {
                            if let Some(output_val) = fmu_task.get_output_value("output_var") {
                                if let Err(e) = logger.write_record(&[
                                    &format!("{:?}", current_time.as_secs_f64()),
                                    &format!("{}", output_val),
                                ]) {
                                    eprintln!("Failed to write log record: {}", e);
                                }
                            }
                        }
                    }
                }
                drop(sim_data_guard); // Release the write lock

                if current_time >= simulation_duration {
                    println!("Simulation finished.");
                    // Shut down the web server gracefully
                    let _ = tx.send(());
                    break;
                }
            },
            SimulationStatus::Paused => {
                println!("Simulation Paused at Time: {:?}", current_time);
                // Do nothing, just wait for status change
            },
            SimulationStatus::Stopped => {
                println!("Simulation Stopped.");
                let _ = tx.send(());
                break;
            },
        }
    }

    logger.flush()?;

    // Wait for the server to shut down
    let _ = server_handle.await;
    Ok(())
}