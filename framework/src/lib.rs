use tokio::time::{self, Duration};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::toposort;
use std::collections::HashMap;

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

// Represents the simulation graph with tasks and their dependencies
pub struct SimulationGraph {
    graph: DiGraph<Box<dyn SimulationTask>, String>,
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

    pub fn add_dependency(&mut self, from_task_name: &str, to_task_name: &str, data_flow: String) -> Result<(), String> {
        let &from_index = self.task_indices.get(from_task_name).ok_or(format!("Task {} not found", from_task_name))?;
        let &to_index = self.task_indices.get(to_task_name).ok_or(format!("Task {} not found", to_task_name))?;
        self.graph.add_edge(from_index, to_index, data_flow);
        Ok(())
    }

    pub fn get_execution_order(&self) -> Result<Vec<NodeIndex>, String> {
        toposort(&self.graph, None).map_err(|e| format!("Causal loop detected: {:?}", e.node_id()))
    }
}

pub async fn run_framework() {
    println!("Framework is running...");

    let simulation_duration = Duration::from_secs(10); // Simulate for 10 seconds
    let time_step = Duration::from_millis(100); // 100ms time step

    let mut interval = time::interval(time_step);
    let mut current_time = Duration::from_secs(0);

    println!("Starting simulation loop...");

    let mut sim_graph = SimulationGraph::new();

    // Example: Add some placeholder tasks
    sim_graph.add_task("FMU1".to_string(), Box::new(FmuTask::new("FMU1".to_string())));
    sim_graph.add_task("FMU2".to_string(), Box::new(FmuTask::new("FMU2".to_string())));

    // Example: Add a dependency
    if let Err(e) = sim_graph.add_dependency("FMU1", "FMU2", "output_to_input".to_string()) {
        eprintln!("Error adding dependency: {}", e);
        return;
    }

    let execution_order = match sim_graph.get_execution_order() {
        Ok(order) => order,
        Err(e) => {
            eprintln!("Error getting execution order: {}", e);
            return;
        }
    };

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
