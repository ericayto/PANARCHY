use std::error::Error;
use std::fmt;
use std::path::PathBuf;
use std::time::Instant;

use crate::config::Scenario;
use crate::snapshot::{SnapshotConfig, SnapshotError, SnapshotManager};
use crate::systems::{
    BookkeepingMetrics, BookkeepingSystem, EnvironmentSystem, PopulationSystem, System,
};
use crate::world::WorldState;

pub struct EngineConfig {
    pub snapshot: SnapshotConfig,
}

impl EngineConfig {
    pub fn from_scenario(scenario: &Scenario) -> Self {
        Self {
            snapshot: scenario.snapshot.clone(),
        }
    }

    pub fn with_snapshot_dir(mut self, dir: String) -> Self {
        self.snapshot.output_dir = dir;
        self
    }
}

pub struct Engine {
    world: WorldState,
    scheduler: Scheduler,
    rng: DeterministicRng,
    tick: u64,
    tick_days: f64,
    scenario_name: String,
    snapshot_manager: SnapshotManager,
}

impl Engine {
    pub fn from_scenario(scenario: &Scenario, config: EngineConfig) -> Result<Self, EngineError> {
        let world = scenario.build_world();
        let mut scheduler = Scheduler::default();
        scheduler.add_system(Box::new(EnvironmentSystem::default()));
        scheduler.add_system(Box::new(PopulationSystem::default()));
        scheduler.add_system(Box::new(BookkeepingSystem::default()));

        Ok(Self {
            world,
            scheduler,
            rng: DeterministicRng::new(scenario.random_seed),
            tick: 0,
            tick_days: scenario.tick_days,
            scenario_name: scenario.name.clone(),
            snapshot_manager: SnapshotManager::new(config.snapshot),
        })
    }

    pub fn tick(&mut self) -> Result<TickSummary, EngineError> {
        self.tick += 1;
        let tick_number = self.tick;
        let system_reports =
            self.scheduler
                .run(&mut self.world, &mut self.rng, tick_number, self.tick_days);
        let metrics = self
            .scheduler
            .get_system::<BookkeepingSystem>()
            .and_then(|system| system.latest_metrics());
        let snapshot_path =
            self.snapshot_manager
                .maybe_snapshot(tick_number, &self.scenario_name, &self.world)?;

        Ok(TickSummary {
            tick: tick_number,
            system_reports,
            metrics,
            snapshot_path,
        })
    }

    pub fn current_tick(&self) -> u64 {
        self.tick
    }

    pub fn scenario_name(&self) -> &str {
        &self.scenario_name
    }

    pub fn total_population(&self) -> u64 {
        self.world.total_population()
    }

    pub fn world(&self) -> &WorldState {
        &self.world
    }
}

#[derive(Clone, Debug)]
pub struct SystemRunReport {
    pub name: &'static str,
    pub duration_ms: f64,
}

#[derive(Clone, Debug)]
pub struct TickSummary {
    pub tick: u64,
    pub system_reports: Vec<SystemRunReport>,
    pub metrics: Option<BookkeepingMetrics>,
    pub snapshot_path: Option<PathBuf>,
}

#[derive(Default)]
pub struct Scheduler {
    systems: Vec<Box<dyn System>>,
}

impl Scheduler {
    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
    }

    pub fn run(
        &mut self,
        world: &mut WorldState,
        rng: &mut DeterministicRng,
        tick: u64,
        dt_days: f64,
    ) -> Vec<SystemRunReport> {
        let mut reports = Vec::with_capacity(self.systems.len());
        for system in self.systems.iter_mut() {
            let start = Instant::now();
            system.update(world, rng, tick, dt_days);
            let elapsed = start.elapsed();
            reports.push(SystemRunReport {
                name: system.name(),
                duration_ms: elapsed.as_secs_f64() * 1_000.0,
            });
        }
        reports
    }

    pub fn get_system<T: 'static>(&self) -> Option<&T> {
        self.systems
            .iter()
            .find_map(|system| system.as_any().downcast_ref::<T>())
    }
}

#[derive(Debug)]
pub enum EngineError {
    Snapshot(SnapshotError),
}

impl From<SnapshotError> for EngineError {
    fn from(value: SnapshotError) -> Self {
        EngineError::Snapshot(value)
    }
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::Snapshot(err) => write!(f, "{err}"),
        }
    }
}

impl Error for EngineError {}

#[derive(Clone, Debug)]
pub struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        const A: u64 = 6364136223846793005;
        const C: u64 = 1;
        self.state = self.state.wrapping_mul(A).wrapping_add(C);
        (self.state >> 32) as u32
    }

    pub fn next_f32(&mut self) -> f32 {
        let value = self.next_u32() as f64 / u32::MAX as f64;
        value as f32
    }

    pub fn gen_range(&mut self, min: f32, max: f32) -> f32 {
        let fraction = self.next_f32();
        min + (max - min) * fraction
    }
}
