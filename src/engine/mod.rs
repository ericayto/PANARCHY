use std::path::PathBuf;

use anyhow::Result;

use crate::{
    rng::{RngManager, SystemRng},
    snapshot::SnapshotWriter,
    world::World,
};

pub struct EngineSettings {
    pub scenario_name: String,
    pub seed: u64,
    pub snapshot_interval_ticks: u64,
    pub snapshot_dir: PathBuf,
}

pub struct EngineBuilder {
    settings: EngineSettings,
    systems: Vec<Box<dyn System>>,
}

impl EngineBuilder {
    pub fn new(settings: EngineSettings) -> Self {
        Self {
            settings,
            systems: Vec::new(),
        }
    }

    pub fn with_system(mut self, system: impl System + 'static) -> Self {
        self.systems.push(Box::new(system));
        self
    }

    pub fn push_system(&mut self, system: impl System + 'static) {
        self.systems.push(Box::new(system));
    }

    pub fn build(self) -> Engine {
        Engine {
            rng: RngManager::new(self.settings.seed),
            systems: self.systems,
            snapshot_writer: SnapshotWriter::new(
                &self.settings.snapshot_dir,
                self.settings.snapshot_interval_ticks,
            ),
            settings: self.settings,
        }
    }
}

pub struct Engine {
    rng: RngManager,
    systems: Vec<Box<dyn System>>,
    snapshot_writer: SnapshotWriter,
    settings: EngineSettings,
}

impl Engine {
    pub fn run(&mut self, world: &mut World, ticks: u64) -> Result<()> {
        for _ in 0..ticks {
            let current_tick = world.tick();
            for system in &mut self.systems {
                let mut rng_stream = self.rng.stream(system.name());
                let ctx = SystemContext {
                    tick: current_tick,
                    dt_days: world.dt_days(),
                    scenario_name: &self.settings.scenario_name,
                };
                system.run(&ctx, world, &mut rng_stream)?;
            }
            world.advance_time();
            self.snapshot_writer
                .maybe_write(world, &self.settings.scenario_name)?;
        }
        Ok(())
    }
}

pub struct SystemContext<'a> {
    pub tick: u64,
    pub dt_days: f64,
    pub scenario_name: &'a str,
}

pub trait System {
    fn name(&self) -> &str;
    fn run(
        &mut self,
        ctx: &SystemContext,
        world: &mut World,
        rng: &mut SystemRng<'_>,
    ) -> Result<()>;
}
