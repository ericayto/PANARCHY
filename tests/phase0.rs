use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use panarchy::{Engine, EngineConfig, Scenario};

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    dir.push(format!("{}_{}", prefix, nanos));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn scenario_loads_and_validates() {
    let scenario = Scenario::load_from_path("scenarios/tiny_island.yaml").expect("scenario loads");
    assert_eq!(scenario.name, "tiny_island");
    assert_eq!(scenario.total_population(), 50_000);
    assert!(scenario.world.tiles.len() >= 2);
}

#[test]
fn engine_is_deterministic_for_same_seed() {
    let scenario = Scenario::load_from_path("scenarios/tiny_island.yaml").unwrap();

    let temp_a = unique_temp_dir("panarchy_a");
    let temp_b = unique_temp_dir("panarchy_b");

    let config_a = EngineConfig::from_scenario(&scenario)
        .with_snapshot_dir(temp_a.to_string_lossy().to_string());
    let config_b = EngineConfig::from_scenario(&scenario)
        .with_snapshot_dir(temp_b.to_string_lossy().to_string());

    let mut engine_a = Engine::from_scenario(&scenario, config_a).unwrap();
    let mut engine_b = Engine::from_scenario(&scenario, config_b).unwrap();

    let mut totals_a = Vec::new();
    let mut totals_b = Vec::new();

    for _ in 0..10 {
        let summary_a = engine_a.tick().unwrap();
        let summary_b = engine_b.tick().unwrap();
        totals_a.push(summary_a.metrics.unwrap().total_population);
        totals_b.push(summary_b.metrics.unwrap().total_population);
    }

    assert_eq!(totals_a, totals_b);
}

#[test]
fn engine_writes_snapshots_on_interval() {
    let scenario = Scenario::load_from_path("scenarios/tiny_island.yaml").unwrap();
    let temp_dir = unique_temp_dir("panarchy_snapshots");
    let config = EngineConfig::from_scenario(&scenario)
        .with_snapshot_dir(temp_dir.to_string_lossy().to_string());
    let mut engine = Engine::from_scenario(&scenario, config).unwrap();

    for _ in 0..6 {
        engine.tick().unwrap();
    }

    let snapshot_file = temp_dir.join(scenario.name).join("tick_000005.json");
    assert!(snapshot_file.exists(), "expected snapshot at tick 5");

    let contents = fs::read_to_string(snapshot_file).unwrap();
    assert!(contents.contains("\"tick\": 5"));
}
