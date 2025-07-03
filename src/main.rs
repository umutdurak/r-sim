use clap::{Parser, Subcommand};
use framework;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run the simulation
    Run {
        /// Total duration of the simulation in seconds.
        #[clap(short, long, default_value_t = 10)]
        simulation_duration_secs: u64,

        /// Simulation time step in milliseconds.
        #[clap(short, long, default_value_t = 100)]
        time_step_millis: u64,

        /// Path to the simulation configuration TOML file.
        #[clap(short, long, value_name = "FILE")]
        config_file: Option<PathBuf>,
    },
    /// Control the simulation status (start, pause, resume, stop)
    Control {
        #[clap(subcommand)]
        command: ControlCommands,
    },
    /// Manage simulation scenarios
    Scenario {
        #[clap(subcommand)]
        command: ScenarioCommands,
    },
}

#[derive(Subcommand, Debug)]
enum ScenarioCommands {
    /// Save the current configuration as a scenario
    Save {
        /// The name of the scenario to save.
        #[clap(value_name = "NAME")]
        name: String,
        /// Path to the configuration file to save.
        #[clap(short, long, value_name = "FILE")]
        config_file: PathBuf,
    },
    /// Load a scenario and run the simulation
    Load {
        /// The name of the scenario to load.
        #[clap(value_name = "NAME")]
        name: String,
        /// Total duration of the simulation in seconds.
        #[clap(short, long, default_value_t = 10)]
        simulation_duration_secs: u64,

        /// Simulation time step in milliseconds.
        #[clap(short, long, default_value_t = 100)]
        time_step_millis: u64,
    },
    /// List all available scenarios
    List,
}
}

#[derive(Subcommand, Debug)]
enum ControlCommands {
    /// Start the simulation
    Start,
    /// Pause the simulation
    Pause,
    /// Resume the simulation
    Resume,
    /// Stop the simulation
    Stop,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Run { simulation_duration_secs, time_step_millis, config_file }) => {
            println!("r-sim: Real-Time Co-Simulation Framework");
            println!("Simulation Duration: {}s", simulation_duration_secs);
            println!("Time Step: {}ms", time_step_millis);
            if let Some(config_path) = &config_file {
                println!("Config File: {}", config_path.display());
            } else {
                println!("Using default embedded configuration.");
            }

            let sim_graph_arc = std::sync::Arc::new(tokio::sync::RwLock::new(framework::SimulationGraph::new()));

            let (status_tx, simulation_data, server_handle) = framework::start_web_server(sim_graph_arc.clone()).await?;

            let simulation_task = framework::run_framework(
                simulation_duration_secs,
                time_step_millis,
                config_file,
                status_tx.subscribe(),
                simulation_data,
                sim_graph_arc.clone(),
            );

            let (simulation_result, server_result) = tokio::join!(simulation_task, server_handle);

            if let Err(e) = simulation_result {
                eprintln!("Framework error: {}", e);
                if matches!(e, framework::FrameworkError::ConfigurationError(_)) {
                    eprintln!("Please provide a configuration file using the -c or --config-file option.");
                    eprintln!("You can use the 'default_config.toml' as a template.");
                }
                std::process::exit(1);
            }

            if let Err(e) = server_result {
                eprintln!("Web server crashed: {:?}", e);
                std::process::exit(1);
            }
        },
        Some(Commands::Control { command }) => {
            let sim_command = match command {
                ControlCommands::Start => framework::SimulationStatus::Running,
                ControlCommands::Pause => framework::SimulationStatus::Paused,
                ControlCommands::Resume => framework::SimulationStatus::Running,
                ControlCommands::Stop => framework::SimulationStatus::Stopped,
            };
            if let Err(e) = framework::send_control_command(sim_command.clone()).await {
                eprintln!("Failed to send control command: {}", e);
                std::process::exit(1);
            }
            println!("Control command sent: {:?}", sim_command);
        },
        Some(Commands::Scenario { command }) => {
            match command {
                ScenarioCommands::Save { name, config_file } => {
                    let scenarios_dir = PathBuf::from("scenarios");
                    if !scenarios_dir.exists() {
                        std::fs::create_dir_all(&scenarios_dir)?;
                    }
                    let scenario_path = scenarios_dir.join(format!("{}.toml", name));
                    std::fs::copy(config_file, &scenario_path)?;
                    println!("Scenario '{}' saved to {}", name, scenario_path.display());
                },
                ScenarioCommands::Load { name, simulation_duration_secs, time_step_millis } => {
                    let scenario_path = PathBuf::from("scenarios").join(format!("{}.toml", name));
                    if !scenario_path.exists() {
                        eprintln!("Scenario '{}' not found.", name);
                        std::process::exit(1);
                    }
                    println!("Loading scenario: {}", name);
                    let config_file = Some(scenario_path);

                    println!("r-sim: Real-Time Co-Simulation Framework");
                    println!("Simulation Duration: {}s", simulation_duration_secs);
                    println!("Time Step: {}ms", time_step_millis);
                    if let Some(config_path) = &config_file {
                        println!("Config File: {}", config_path.display());
                    } else {
                        println!("Using default embedded configuration.");
                    }

                    let sim_graph_arc = std::sync::Arc::new(tokio::sync::RwLock::new(framework::SimulationGraph::new()));

                    let (status_tx, simulation_data, server_handle) = framework::start_web_server(sim_graph_arc.clone()).await?;

                    let simulation_task = framework::run_framework(
                        simulation_duration_secs,
                        time_step_millis,
                        config_file,
                        status_tx.subscribe(),
                        simulation_data,
                        sim_graph_arc.clone(),
                    );

                    let (simulation_result, server_result) = tokio::join!(simulation_task, server_handle);

                    if let Err(e) = simulation_result {
                        eprintln!("Framework error: {}", e);
                        if matches!(e, framework::FrameworkError::ConfigurationError(_)) {
                            eprintln!("Please provide a configuration file using the -c or --config-file option.");
                            eprintln!("You can use the 'default_config.toml' as a template.");
                        }
                        std::process::exit(1);
                    }

                    if let Err(e) = server_result {
                        eprintln!("Web server crashed: {:?}", e);
                        std::process::exit(1);
                    }
                },
                ScenarioCommands::List => {
                    let scenarios_dir = PathBuf::from("scenarios");
                    if !scenarios_dir.exists() {
                        println!("No scenarios found.");
                        return Ok(());
                    }
                    println!("Available scenarios:");
                    for entry in std::fs::read_dir(scenarios_dir)? {
                        let entry = entry?;
                        let path = entry.path();
                        if let Some(extension) = path.extension() {
                            if extension == "toml" {
                                if let Some(name) = path.file_stem() {
                                    println!("- {}", name.to_string_lossy());
                                }
                            }
                        }
                    }
                },
            }
        }
        None => {
            println!("No command provided. Use `r-sim --help` for more information.");
        }
    }
    Ok(())
}