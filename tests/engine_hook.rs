use panarchy::{
    engine::{EngineBuilder, EngineSettings},
    scenario::ScenarioLoader,
    systems::{
        BookkeepingSystem, EconomySystem, EnvironmentSystem, FinanceSystem, InfrastructureSystem,
        PolicySystem, PopulationSystem, TechnologySystem,
    },
};
use tempfile::tempdir;

#[test]
fn engine_runs_hook_each_tick() {
    let loader = ScenarioLoader::new(".");
    let scenario = loader
        .load("scenarios/tiny_island.yaml")
        .expect("scenario should load");
    let mut world = scenario.build_world();
    let temp = tempdir().expect("tempdir");
    let settings = EngineSettings {
        scenario_name: scenario.name.clone(),
        seed: scenario.seed,
        snapshot_interval_ticks: 0,
        snapshot_dir: temp.path().to_path_buf(),
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

    let mut ticks = Vec::new();
    engine
        .run_with_hook(&mut world, 6, |snapshot| ticks.push(snapshot.tick))
        .expect("run succeeds");

    assert_eq!(ticks.len(), 6);
    assert_eq!(ticks.first().copied(), Some(1));
    assert_eq!(ticks.last().copied(), Some(6));
}
