use crate::engine::DeterministicRng;
use crate::world::WorldState;

use super::System;

#[derive(Debug)]
pub struct PopulationSystem {
    pub annual_birth_rate: f32,
    pub annual_death_rate: f32,
    pub volatility: f32,
}

impl Default for PopulationSystem {
    fn default() -> Self {
        Self {
            annual_birth_rate: 0.013,
            annual_death_rate: 0.010,
            volatility: 0.001,
        }
    }
}

impl System for PopulationSystem {
    fn name(&self) -> &'static str {
        "population"
    }

    fn update(
        &mut self,
        world: &mut WorldState,
        rng: &mut DeterministicRng,
        tick: u64,
        dt_days: f64,
    ) {
        let dt_years = (dt_days / 365.0) as f32;
        for group in &mut world.populations {
            let births = group.count as f32 * self.annual_birth_rate * dt_years;
            let deaths = group.count as f32 * self.annual_death_rate * dt_years;
            let noise = rng.gen_range(-self.volatility, self.volatility) * group.count as f32;
            let new_count = (group.count as f32 + births - deaths + noise)
                .max(0.0)
                .round();
            group.count = new_count as u32;
            group.mean_age_years = (group.mean_age_years + dt_years).min(120.0);
        }

        let _ = tick;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
