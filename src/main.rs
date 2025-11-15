use std::{env, error::Error, path::PathBuf, process};

use panarchy::{Engine, EngineConfig, Scenario};

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let scenario_path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("scenarios/tiny_island.yaml"));

    let scenario = Scenario::load_from_path(&scenario_path)?;
    println!(
        "Loaded scenario '{}' with total population {}",
        scenario.name,
        scenario.total_population()
    );

    let config = EngineConfig::from_scenario(&scenario);
    let mut engine = Engine::from_scenario(&scenario, config)?;

    for _ in 0..scenario.ticks {
        let summary = engine.tick()?;
        if let Some(metrics) = summary.metrics {
            println!(
                "Tick {:>4} | total population {:>8}",
                summary.tick, metrics.total_population
            );
        }
    }

    println!("Simulation finished at tick {}", engine.current_tick());
    Ok(())
}
