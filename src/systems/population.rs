use anyhow::Result;
use rand::Rng;

use crate::{
    engine::{System, SystemContext},
    rng::SystemRng,
    world::{EntityId, World},
};

pub struct PopulationSystem;

impl PopulationSystem {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PopulationSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for PopulationSystem {
    fn name(&self) -> &str {
        "population"
    }

    fn run(
        &mut self,
        ctx: &SystemContext,
        world: &mut World,
        rng: &mut SystemRng<'_>,
    ) -> Result<()> {
        world.bookkeeping.starving_regions.clear();
        let mut ids: Vec<EntityId> = world.populations.keys().cloned().collect();
        ids.sort();
        for id in ids {
            let region_name = world
                .regions
                .get(&id)
                .map(|r| r.name.clone())
                .unwrap_or_else(|| "unknown".into());
            let population = world
                .populations
                .get_mut(&id)
                .expect("population component should exist");
            let stock = world
                .resources
                .get_mut(&id)
                .expect("resource component should exist");
            let dt_years = ctx.dt_days / 365.0;
            let births = (population.citizens as f64 * population.annual_birth_rate * dt_years)
                .round() as i64;
            let deaths = (population.citizens as f64 * population.annual_death_rate * dt_years)
                .round() as i64;
            let mut net_delta = births - deaths;

            let consumption =
                population.citizens as f64 * population.food_consumption_per_capita * ctx.dt_days;
            if stock.food >= consumption {
                stock.food -= consumption;
            } else if consumption > 0.0 {
                let shortfall = consumption - stock.food;
                stock.food = 0.0;
                let shortage_ratio = (shortfall / consumption).clamp(0.0, 1.0);
                let starvation = (population.citizens as f64 * shortage_ratio * 0.02).ceil() as i64; // starvation penalty
                net_delta -= starvation;
                world.bookkeeping.starving_regions.push(region_name.clone());
            }

            let shock: f64 = rng.gen_range(-0.0005..0.0005);
            let adjusted_rate = (population.target_employment_rate + shock).clamp(0.0, 1.0);
            let employed = (population.citizens as f64 * adjusted_rate).round() as u64;

            let next_citizens = (population.citizens as i64 + net_delta).max(0) as u64;
            population.citizens = next_citizens;
            population.employed = employed.min(population.citizens);
        }
        Ok(())
    }
}
