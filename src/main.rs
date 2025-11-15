use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use panarchy::{
    engine::{EngineBuilder, EngineSettings},
    scenario::ScenarioLoader,
    systems::{
        BookkeepingSystem, EconomySystem, EnvironmentSystem, FinanceSystem, InfrastructureSystem,
        PolicySystem, PopulationSystem, TechnologySystem,
    },
};

#[derive(Debug, Parser)]
#[command(author, version, about = "PANARCHY Phase 0 runner")]
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
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let loader = ScenarioLoader::new(".");
    let scenario = loader.load(&cli.scenario)?;
    let mut world = scenario.build_world();
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

    let mut engine = EngineBuilder::new(settings)
        .with_system(EnvironmentSystem::new())
        .with_system(InfrastructureSystem::new())
        .with_system(PopulationSystem::new())
        .with_system(EconomySystem::new())
        .with_system(FinanceSystem::new())
        .with_system(PolicySystem::new())
        .with_system(TechnologySystem::new())
        .with_system(BookkeepingSystem::new())
        .build();

    engine.run(&mut world, ticks)?;
    println!(
        "Scenario '{}' completed for {} ticks. Final population: {}",
        scenario.name,
        ticks,
        world.total_population()
    );
    Ok(())
}
