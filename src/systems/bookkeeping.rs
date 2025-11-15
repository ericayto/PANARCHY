use crate::engine::DeterministicRng;
use crate::world::WorldState;

use super::System;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BookkeepingMetrics {
    pub total_population: u64,
    pub tick: u64,
}

#[derive(Default)]
pub struct BookkeepingSystem {
    latest: Option<BookkeepingMetrics>,
}

impl BookkeepingSystem {
    pub fn latest_metrics(&self) -> Option<BookkeepingMetrics> {
        self.latest
    }
}

impl System for BookkeepingSystem {
    fn name(&self) -> &'static str {
        "bookkeeping"
    }

    fn update(
        &mut self,
        world: &mut WorldState,
        _rng: &mut DeterministicRng,
        tick: u64,
        _dt_days: f64,
    ) {
        self.latest = Some(BookkeepingMetrics {
            total_population: world.total_population(),
            tick,
        });
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
