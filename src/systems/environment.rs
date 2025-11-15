use anyhow::Result;
use rand::Rng;

use crate::{
    engine::{System, SystemContext},
    rng::SystemRng,
    world::World,
};

pub struct EnvironmentSystem;

impl EnvironmentSystem {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EnvironmentSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for EnvironmentSystem {
    fn name(&self) -> &str {
        "environment"
    }

    fn run(
        &mut self,
        ctx: &SystemContext,
        world: &mut World,
        rng: &mut SystemRng<'_>,
    ) -> Result<()> {
        let dt = ctx.dt_days;
        let mut ids: Vec<_> = world.regions.keys().cloned().collect();
        ids.sort();
        for id in ids {
            if let (Some(region), Some(pop), Some(stock)) = (
                world.regions.get(&id),
                world.populations.get(&id),
                world.resources.get_mut(&id),
            ) {
                let thousands = (pop.citizens as f64 / 1_000.0).max(0.1);
                let fluctuation: f64 = rng.gen_range(0.95..1.05);
                let food_gain = region.food_regen_per_1000 * thousands * dt * fluctuation;
                let energy_gain = region.energy_regen_per_1000 * thousands * dt * fluctuation;
                stock.food += food_gain.max(0.0);
                stock.energy += energy_gain.max(0.0);
            }
        }
        Ok(())
    }
}
