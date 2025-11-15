use std::any::Any;

use crate::engine::DeterministicRng;
use crate::world::WorldState;

pub mod bookkeeping;
pub mod environment;
pub mod population;

pub use bookkeeping::{BookkeepingMetrics, BookkeepingSystem};
pub use environment::EnvironmentSystem;
pub use population::PopulationSystem;

pub trait System: Send {
    fn name(&self) -> &'static str;
    fn update(
        &mut self,
        world: &mut WorldState,
        rng: &mut DeterministicRng,
        tick: u64,
        dt_days: f64,
    );
    fn as_any(&self) -> &dyn Any;
}
