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
        snapshot_dir: std::path::PathBuf::from("snapshots_phase2_tests"),
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
fn loans_expand_when_deposits_exhausted() {
    let loader = scenario_loader();
    let scenario = loader.load(scenario_path()).unwrap();
    let mut world = scenario.build_world();
    let id = world
        .entity_ids()
        .into_iter()
        .next()
        .expect("region exists");
    if let Some(finance) = world.finance_mut(id) {
        finance.bank_deposits = 0.0;
        finance.loan_balance = 0.0;
    }
    if let Some(econ) = world.economy_mut(id) {
        econ.propensity_to_consume = 0.0;
    }
    let mut engine = build_engine(scenario.seed).build();
    engine.run(&mut world, 1).unwrap();
    let finance = world.finance(id).expect("finance component exists");
    assert!(
        finance.loan_balance > 0.0,
        "loan balance should expand when deposits are empty"
    );
}

#[test]
fn transport_capacity_can_create_shortfall() {
    let loader = scenario_loader();
    let scenario = loader.load(scenario_path()).unwrap();
    let mut world = scenario.build_world();
    let id = world
        .entity_ids()
        .into_iter()
        .next()
        .expect("region exists");
    if let Some(infra) = world.infrastructure_mut(id) {
        infra.transport_capacity = 100.0;
    }
    let mut engine = build_engine(scenario.seed).build();
    engine.run(&mut world, 1).unwrap();
    let economy = world.economy(id).expect("economy component exists");
    assert!(
        economy.transport_shortfall > 0.0,
        "limited transport should introduce a delivery shortfall"
    );
}

#[test]
fn infrastructure_investments_raise_capacity() {
    let loader = scenario_loader();
    let scenario = loader.load(scenario_path()).unwrap();
    let mut world = scenario.build_world();
    let id = world
        .entity_ids()
        .into_iter()
        .next()
        .expect("region exists");
    if let Some(econ) = world.economy_mut(id) {
        econ.wage = 50.0;
        econ.basic_income_per_capita = 2000.0;
        econ.propensity_to_consume = 1.0;
        econ.job_matching_efficiency = 0.1;
    }
    if let Some(finance) = world.finance_mut(id) {
        finance.bank_deposits = 0.0;
        finance.loan_balance = 0.0;
        finance.infrastructure_spend_fraction = 0.4;
    }
    if let Some(infra) = world.infrastructure_mut(id) {
        infra.degradation_rate = 0.0;
        infra.maintenance_cost = 0.0;
    }
    let baseline_capacity = world
        .infrastructure(id)
        .expect("infra component exists")
        .power_capacity;
    let mut engine = build_engine(scenario.seed).build();
    engine.run(&mut world, 6).unwrap();
    let updated_capacity = world
        .infrastructure(id)
        .expect("infra component exists")
        .power_capacity;
    assert!(
        updated_capacity > baseline_capacity,
        "investments should grow power capacity ({} -> {})",
        baseline_capacity,
        updated_capacity
    );
}
