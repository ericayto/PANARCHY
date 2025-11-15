use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use panarchy::{
    engine::{Engine, EngineBuilder, EngineSettings},
    scenario::ScenarioLoader,
    systems::{
        BookkeepingSystem, EconomySystem, EnvironmentSystem, FinanceSystem, InfrastructureSystem,
        PolicySystem, PopulationSystem, TechnologySystem,
    },
    web::{self, WebServerConfig},
};

#[derive(Debug, Clone, Parser)]
#[command(author, version, about = "PANARCHY runner with interactive UI mode")]
struct Cli {
    /// Path to the scenario YAML file
    #[arg(long, default_value = "scenarios/tiny_island.yaml")]
    scenario: PathBuf,

    /// Override tick count (uses scenario default when omitted)
    #[arg(long)]
    ticks: Option<u64>,

    /// Override snapshot interval in ticks
    #[arg(long)]
    snapshot_interval: Option<u64>,

    /// Directory for snapshots
    #[arg(long)]
    snapshot_dir: Option<PathBuf>,

    /// Run the modern web UI and stream the simulation live
    #[arg(long)]
    web: bool,

    /// Host/IP the UI server should bind to
    #[arg(long, default_value = "127.0.0.1")]
    web_host: String,

    /// Port for the UI server
    #[arg(long, default_value_t = 8080)]
    web_port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    if cli.web {
        run_with_ui(cli).await
    } else {
        run_headless(cli)
    }
}

fn run_headless(cli: Cli) -> Result<()> {
    let loader = ScenarioLoader::new(".");
    let scenario = loader.load(&cli.scenario)?;
    let ticks = scenario.ticks(cli.ticks);
    let snapshot_interval = cli
        .snapshot_interval
        .unwrap_or(scenario.snapshot_interval_ticks);
    let snapshot_dir = cli
        .snapshot_dir
        .unwrap_or_else(|| PathBuf::from("snapshots"));
    let settings = EngineSettings {
        scenario_name: scenario.name.clone(),
        seed: scenario.seed,
        snapshot_interval_ticks: snapshot_interval,
        snapshot_dir,
    };

    let mut engine = build_engine(settings);
    let mut world = scenario.build_world();
    engine.run(&mut world, ticks)?;
    println!(
        "Scenario '{}' completed for {} ticks. Final population: {}",
        scenario.name,
        ticks,
        world.total_population()
    );
    Ok(())
}

async fn run_with_ui(cli: Cli) -> Result<()> {
    let loader = ScenarioLoader::new(".");
    let scenario = loader.load(&cli.scenario)?;
    let ticks = scenario.ticks(cli.ticks);
    let snapshot_interval = cli
        .snapshot_interval
        .unwrap_or(scenario.snapshot_interval_ticks);
    let snapshot_dir = cli
        .snapshot_dir
        .unwrap_or_else(|| PathBuf::from("snapshots"));

    let config = WebServerConfig {
        scenario,
        ticks,
        snapshot_interval,
        snapshot_dir,
        host: cli.web_host,
        port: cli.web_port,
    };
    web::run(config).await
}

fn build_engine(settings: EngineSettings) -> Engine {
    EngineBuilder::new(settings)
        .with_system(EnvironmentSystem::new())
        .with_system(InfrastructureSystem::new())
        .with_system(PopulationSystem::new())
        .with_system(EconomySystem::new())
        .with_system(FinanceSystem::new())
        .with_system(PolicySystem::new())
        .with_system(TechnologySystem::new())
        .with_system(BookkeepingSystem::new())
        .build()
}
