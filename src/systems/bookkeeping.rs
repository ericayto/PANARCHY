use anyhow::Result;

use crate::{
    engine::{System, SystemContext},
    rng::SystemRng,
    world::World,
};

pub struct BookkeepingSystem;

impl BookkeepingSystem {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BookkeepingSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for BookkeepingSystem {
    fn name(&self) -> &str {
        "bookkeeping"
    }

    fn run(
        &mut self,
        _ctx: &SystemContext,
        world: &mut World,
        _rng: &mut SystemRng<'_>,
    ) -> Result<()> {
        for stock in world.resources.values_mut() {
            stock.clamp_non_negative();
        }
        world.bookkeeping.starving_regions.sort();
        world.bookkeeping.starving_regions.dedup();
        Ok(())
    }
}
