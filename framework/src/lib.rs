use tokio::time::{self, Duration};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::toposort;
use std::collections::HashMap;
use serde::Deserialize;

// Define a trait for simulation tasks
pub trait SimulationTask: Send + Sync + 'static {
    fn execute(&mut self, current_time: Duration);
    fn get_inputs(&self) -> Vec<String>;
    fn get_outputs(&self) -> Vec<String>;
    fn set_input(&mut self, name: &str, value: f64);
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

impl SimulationTask for FmuTask {
    fn execute(&mut self, current_time: Duration) {
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
}

// Define a trait for I/O tasks
pub trait IoTask: SimulationTask {
    // Specific I/O methods can be added here later
    fn initialize_io(&mut self);
    fn read_io(&mut self);
    fn write_io(&mut self);
}

// Placeholder for GPIO Task
pub struct GpioTask {
    name: String,
    inputs: HashMap<String, f64>,
    outputs: HashMap<String, f64>,
}

impl GpioTask {
    pub fn new(name: String) -> Self {
        GpioTask { 
            name,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
        }
    }
}

impl SimulationTask for GpioTask {
    fn execute(&mut self, _current_time: Duration) {
        println!("  Executing GPIO Task {}", self.name);
        self.read_io();
        // Process inputs if any
        self.write_io();
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
}

impl IoTask for GpioTask {
    fn initialize_io(&mut self) {
        println!("    GPIO Task {} initialized.", self.name);
    }
    fn read_io(&mut self) {
        // Simulate reading GPIO pins
        // println!("    GPIO Task {} reading inputs.", self.name);
    }
    fn write_io(&mut self) {
        // Simulate writing to GPIO pins
        // println!("    GPIO Task {} writing outputs.", self.name);
    }
}

// Placeholder for Serial Communication Task
pub struct SerialTask {
    name: String,
    inputs: HashMap<String, f64>,
    outputs: HashMap<String, f64>,
}

impl SerialTask {
    pub fn new(name: String) -> Self {
        SerialTask { 
            name,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
        }
    }
}

impl SimulationTask for SerialTask {
    fn execute(&mut self, _current_time: Duration) {
        println!("  Executing Serial Task {}", self.name);
        self.read_io();
        // Process inputs if any
        self.write_io();
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
}

impl IoTask for SerialTask {
    fn initialize_io(&mut self) {
        println!("    Serial Task {} initialized.", self.name);
    }
    fn read_io(&mut self) {
        // Simulate reading serial data
        // println!("    Serial Task {} reading inputs.", self.name);
    }
    fn write_io(&mut self) {
        // Simulate writing serial data
        // println!("    Serial Task {} writing outputs.", self.name);
    }
}

// Placeholder for UDP Communication Task
pub struct UdpTask {
    name: String,
    inputs: HashMap<String, f64>,
    outputs: HashMap<String, f64>,
}

impl UdpTask {
    pub fn new(name: String) -> Self {
        UdpTask { 
            name,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
        }
    }
}

impl SimulationTask for UdpTask {
    fn execute(&mut self, _current_time: Duration) {
        println!("  Executing UDP Task {}", self.name);
        self.read_io();
        // Process inputs if any
        self.write_io();
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
}

impl IoTask for UdpTask {
    fn initialize_io(&mut self) {
        println!("    UDP Task {} initialized.", self.name);
    }
    fn read_io(&mut self) {
        // Simulate reading UDP data
        // println!("    UDP Task {} reading inputs.", self.name);
    }
    fn write_io(&mut self) {
        // Simulate writing UDP data
        // println!("    UDP Task {} writing outputs.", self.name);
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

    pub fn add_dependency(&mut self, from_task_name: &str, to_task_name: &str, dep_type: DependencyType) -> Result<(), String> {
        let &from_index = self.task_indices.get(from_task_name).ok_or(format!("Task {} not found", from_task_name))?;
        let &to_index = self.task_indices.get(to_task_name).ok_or(format!("Task {} not found", to_task_name))?;
        self.graph.add_edge(from_index, to_index, dep_type);
        Ok(())
    }

    pub fn get_execution_order(&self) -> Result<Vec<NodeIndex>, String> {
        // Create a graph for topological sort that ignores memory block dependencies
        let mut graph_for_toposort = DiGraph::<(), ()>::new();
        for node_idx in self.graph.node_indices() {
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
            Err(e) => Err(format!("Causal loop detected that cannot be resolved by memory blocks: {:?}", e.node_id())),
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
#[serde(tag = "type")]
pub enum TaskConfig {
    Fmu(FmuConfig),
    Gpio(GpioConfig),
    Serial(SerialConfig),
    Udp(UdpConfig),
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
    simulation_duration_secs: u64,
    time_step_millis: u64,
    tasks: Vec<TaskConfig>,
    dependencies: Vec<DependencyConfig>,
}

pub async fn run_framework() {
    println!("Framework is running...");

    let config_str = r#"
    simulation_duration_secs = 10
    time_step_millis = 100

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

    let config: SimulationConfig = match toml::from_str(config_str) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to parse configuration: {}", e);
            return;
        }
    };

    println!("Loaded Configuration: {:#?}", config);

    let simulation_duration = Duration::from_secs(config.simulation_duration_secs);
    let time_step = Duration::from_millis(config.time_step_millis);

    let mut interval = time::interval(time_step);
    let mut current_time = Duration::from_secs(0);

    println!("Starting simulation loop...");

    let mut sim_graph = SimulationGraph::new();

    // Populate sim_graph from config
    let mut io_tasks_to_initialize: Vec<NodeIndex> = Vec::new();

    for task_config in config.tasks {
        match task_config {
            TaskConfig::Fmu(fmu_cfg) => {
                sim_graph.add_task(fmu_cfg.name.clone(), Box::new(FmuTask::new(fmu_cfg.name)));
            },
            TaskConfig::Gpio(gpio_cfg) => {
                let task = Box::new(GpioTask::new(gpio_cfg.name.clone()));
                sim_graph.add_task(gpio_cfg.name, task);
                // io_tasks_to_initialize.push(sim_graph.task_indices[&gpio_cfg.name]);
            },
            TaskConfig::Serial(serial_cfg) => {
                let task = Box::new(SerialTask::new(serial_cfg.name.clone()));
                sim_graph.add_task(serial_cfg.name, task);
                // io_tasks_to_initialize.push(sim_graph.task_indices[&serial_cfg.name]);
            },
            TaskConfig::Udp(udp_cfg) => {
                let task = Box::new(UdpTask::new(udp_cfg.name.clone()));
                sim_graph.add_task(udp_cfg.name, task);
                // io_tasks_to_initialize.push(sim_graph.task_indices[&udp_cfg.name]);
            },
        }
    }

    for dep_config in config.dependencies {
        let dep_type = match dep_config.dep_type.as_str() {
            "direct" => DependencyType::Direct(dep_config.data_flow),
            "memory_block" => DependencyType::MemoryBlock(dep_config.data_flow),
            _ => {
                eprintln!("Unknown dependency type: {}", dep_config.dep_type);
                return;
            }
        };
        if let Err(e) = sim_graph.add_dependency(&dep_config.from, &dep_config.to, dep_type) {
            eprintln!("Error adding dependency: {}", e);
            return;
        }
    }

    let execution_order = match sim_graph.get_execution_order() {
        Ok(order) => order,
        Err(e) => {
            eprintln!("Error getting execution order: {}", e);
            return;
        }
    };

    // Initialize I/O tasks (moved here after graph population)
    for node_index in &execution_order {
        let task = &mut sim_graph.graph[*node_index];
        // Check if the task is an IoTask and initialize it
        // This requires downcasting, which is complex with Box<dyn Trait>
        // For now, we'll skip explicit initialize_io calls here and assume initialization
        // happens within the task's constructor or first execute call.
    }

    loop {
        interval.tick().await;
        current_time += time_step;

        println!("Simulation Time: {:?}", current_time);

        for node_index in &execution_order {
            let task = &mut sim_graph.graph[*node_index];
            task.execute(current_time);
        }

        if current_time >= simulation_duration {
            println!("Simulation finished.");
            break;
        }
    }
}
