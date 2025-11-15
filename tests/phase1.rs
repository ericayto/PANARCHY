use panarchy::{
    engine::{EngineBuilder, EngineSettings},
    scenario::ScenarioLoader,
    systems::{
        BookkeepingSystem, EconomySystem, EnvironmentSystem, FinanceSystem, InfrastructureSystem,
        PolicySystem, PopulationSystem, TechnologySystem,
    },
};

fn scenario_loader() -> ScenarioLoader {
    ScenarioLoader::new(env!("CARGO_MANIFEST_DIR"))
}

fn scenario_path() -> std::path::PathBuf {
    std::path::PathBuf::from("scenarios/tiny_island.yaml")
}

fn build_engine(seed: u64) -> EngineBuilder {
    let settings = EngineSettings {
        scenario_name: "tiny_island".into(),
        seed,
        snapshot_interval_ticks: 0,
        snapshot_dir: std::path::PathBuf::from("snapshots_phase1_tests"),
    };
    EngineBuilder::new(settings)
        .with_system(EnvironmentSystem::new())
        .with_system(InfrastructureSystem::new())
        .with_system(PopulationSystem::new())
        .with_system(EconomySystem::new())
        .with_system(FinanceSystem::new())
        .with_system(PolicySystem::new())
        .with_system(TechnologySystem::new())
        .with_system(BookkeepingSystem::new())
}

#[test]
fn employment_rises_when_productivity_drops() {
    let loader = scenario_loader();
    let scenario = loader.load(scenario_path()).unwrap();

    let mut world_high_demand = scenario.build_world();
    for id in world_high_demand.entity_ids() {
        if let Some(econ) = world_high_demand.economy_mut(id) {
            econ.food_productivity_per_worker *= 0.5;
            econ.energy_productivity_per_worker *= 0.5;
        }
    }

    let mut world_low_demand = scenario.build_world();
    for id in world_low_demand.entity_ids() {
        if let Some(econ) = world_low_demand.economy_mut(id) {
            econ.food_productivity_per_worker *= 2.0;
            econ.energy_productivity_per_worker *= 2.0;
        }
    }

    let mut engine_high = build_engine(scenario.seed).build();
    engine_high.run(&mut world_high_demand, 30).unwrap();
    let mut engine_low = build_engine(scenario.seed).build();
    engine_low.run(&mut world_low_demand, 30).unwrap();

    let high_employment: u64 = world_high_demand
        .entity_ids()
        .into_iter()
        .filter_map(|id| world_high_demand.population(id))
        .map(|pop| pop.employed)
        .sum();
    let low_employment: u64 = world_low_demand
        .entity_ids()
        .into_iter()
        .filter_map(|id| world_low_demand.population(id))
        .map(|pop| pop.employed)
        .sum();

    assert!(
        high_employment > low_employment,
        "lower productivity should require more workers ({} vs {})",
        high_employment,
        low_employment
    );
}

#[test]
fn posted_prices_rise_when_food_shortages_persist() {
    let loader = scenario_loader();
    let scenario = loader.load(scenario_path()).unwrap();
    let mut world = scenario.build_world();
    let id = world
        .entity_ids()
        .into_iter()
        .next()
        .expect("region exists");
    if let Some(stock) = world.resources_mut(id) {
        stock.food = 5.0; // force a shortage
    }
    let baseline_price = world.economy(id).unwrap().food_price;

    let mut engine = build_engine(scenario.seed).build();
    engine.run(&mut world, 1).unwrap();

    let updated_price = world.economy(id).unwrap().food_price;
    assert!(
        updated_price > baseline_price,
        "expected food price {} to rise above baseline {}",
        updated_price,
        baseline_price
    );
    let shortage_ratio = world.economy(id).unwrap().food_shortage_ratio;
    assert!(shortage_ratio > 0.0, "shortage signal should be tracked");
}
