use std::path::PathBuf;

use panarchy::{
    engine::{EngineBuilder, EngineSettings},
    scenario::ScenarioLoader,
    systems::{
        BookkeepingSystem, EconomySystem, EnvironmentSystem, FinanceSystem, InfrastructureSystem,
        PopulationSystem,
    },
};

fn scenario_loader() -> ScenarioLoader {
    ScenarioLoader::new(env!("CARGO_MANIFEST_DIR"))
}

fn scenario_path() -> PathBuf {
    PathBuf::from("scenarios/tiny_island.yaml")
}

fn build_engine(seed: u64, snapshot_dir: PathBuf, snapshot_interval: u64) -> EngineBuilder {
    let settings = EngineSettings {
        scenario_name: "tiny_island".into(),
        seed,
        snapshot_interval_ticks: snapshot_interval,
        snapshot_dir,
    };
    EngineBuilder::new(settings)
        .with_system(EnvironmentSystem::new())
        .with_system(InfrastructureSystem::new())
        .with_system(PopulationSystem::new())
        .with_system(EconomySystem::new())
        .with_system(FinanceSystem::new())
        .with_system(BookkeepingSystem::new())
}

#[test]
fn scenario_loader_reads_fixture() {
    let loader = scenario_loader();
    let scenario = loader.load(scenario_path()).expect("scenario parses");
    assert_eq!(scenario.name, "tiny_island");
    assert_eq!(scenario.regions.len(), 3);
    assert_eq!(
        scenario.regions.iter().map(|r| r.citizens).sum::<u64>(),
        50_000
    );
}

#[test]
fn engine_runs_deterministically() {
    let loader = scenario_loader();
    let scenario = loader.load(scenario_path()).unwrap();
    let ticks = 60;

    let mut world_a = scenario.build_world();
    let mut engine_a = build_engine(scenario.seed, PathBuf::from("snapshots_test_a"), 0).build();
    engine_a.run(&mut world_a, ticks).unwrap();

    let mut world_b = scenario.build_world();
    let mut engine_b = build_engine(scenario.seed, PathBuf::from("snapshots_test_b"), 0).build();
    engine_b.run(&mut world_b, ticks).unwrap();

    assert_eq!(world_a.total_population(), world_b.total_population());
}

#[test]
fn engine_emits_snapshots() {
    let loader = scenario_loader();
    let scenario = loader.load(scenario_path()).unwrap();
    let ticks = 30;
    let temp_dir = tempfile::tempdir().unwrap();
    let snapshot_dir = temp_dir.path().join("snaps");

    let mut world = scenario.build_world();
    let mut engine = build_engine(scenario.seed, snapshot_dir.clone(), 10).build();
    engine.run(&mut world, ticks).unwrap();

    let expected = snapshot_dir.join("tiny_island").join("tick_000010.json");
    assert!(
        expected.exists(),
        "expected snapshot {} to exist",
        expected.display()
    );

    let data = std::fs::read_to_string(expected).unwrap();
    assert!(
        data.contains("\"scenario\": \"tiny_island\""),
        "snapshot should contain scenario metadata"
    );
}
