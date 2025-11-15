use panarchy::{
    engine::{EngineBuilder, EngineSettings},
    scenario::ScenarioLoader,
    systems::{
        BookkeepingSystem, EconomySystem, EnvironmentSystem, FinanceSystem, InfrastructureSystem,
        PolicySystem, PopulationSystem, TechnologySystem,
    },
    world::EntityId,
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
        snapshot_dir: std::path::PathBuf::from("snapshots_phase3_tests"),
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

fn region_named(world: &panarchy::World, name: &str) -> Option<EntityId> {
    world
        .entity_ids()
        .into_iter()
        .find(|id| world.region(*id).map(|r| r.name.as_str()) == Some(name))
}

#[test]
fn rnd_budget_unlocks_tech_and_boosts_productivity() {
    let loader = scenario_loader();
    let scenario = loader.load(scenario_path()).unwrap();
    let mut world = scenario.build_world();
    let id = region_named(&world, "Research Atoll").expect("region exists");
    let baseline_productivity = world
        .technology(id)
        .map(|tech| (tech.base_food_productivity, tech.base_energy_productivity))
        .unwrap();
    if let Some(policy) = world.policy_mut(id) {
        policy.rnd_fraction = 1.0;
        policy.tax_rate = 0.4;
        policy.public_investment_fraction = 0.0;
    }
    if let Some(tech) = world.technology_mut(id) {
        tech.research_efficiency = 4.0;
        tech.baseline_rnd_budget_per_capita = 35.0;
    }
    let mut engine = build_engine(scenario.seed).build();
    engine.run(&mut world, 12).unwrap();
    let tech = world.technology(id).expect("tech component");
    assert!(
        tech.unlocked.len() >= 3,
        "expected multiple tech unlocks, got {:?}",
        tech.unlocked
    );
    let economy = world.economy(id).expect("economy component");
    assert!(
        economy.food_productivity_per_worker > baseline_productivity.0,
        "food productivity should rise from tech"
    );
    assert!(
        economy.energy_productivity_per_worker > baseline_productivity.1,
        "energy productivity should rise from tech"
    );
}

#[test]
fn policy_expands_transfers_when_unemployment_spikes() {
    let loader = scenario_loader();
    let scenario = loader.load(scenario_path()).unwrap();
    let mut world = scenario.build_world();
    let id = region_named(&world, "Harbor Town").expect("region exists");
    let baseline_transfer = world.policy(id).unwrap().transfer_per_capita;
    if let Some(pop) = world.population_mut(id) {
        pop.employed = (pop.citizens as f64 * 0.2) as u64;
    }
    if let Some(policy) = world.policy_mut(id) {
        policy.tax_rate = 0.35;
    }
    let mut engine = build_engine(scenario.seed).build();
    engine.run(&mut world, 3).unwrap();
    let updated_transfer = world.policy(id).unwrap().transfer_per_capita;
    assert!(
        updated_transfer > baseline_transfer,
        "transfer_per_capita should rise when unemployment spikes"
    );
}

#[test]
fn public_investment_allocates_to_infrastructure() {
    let loader = scenario_loader();
    let scenario = loader.load(scenario_path()).unwrap();
    let mut world = scenario.build_world();
    let id = region_named(&world, "Highlands").expect("region exists");
    if let Some(policy) = world.policy_mut(id) {
        policy.public_investment_fraction = 0.6;
        policy.rnd_fraction = 0.0;
        policy.tax_rate = 0.4;
    }
    let baseline_pending = world
        .infrastructure(id)
        .map(|infra| infra.pending_investment)
        .unwrap_or(0.0);
    let mut engine = build_engine(scenario.seed).build();
    engine.run(&mut world, 2).unwrap();
    let pending = world
        .infrastructure(id)
        .map(|infra| infra.pending_investment)
        .unwrap_or(0.0);
    assert!(
        pending > baseline_pending,
        "policy-driven investment should increase pending infra funds"
    );
}
