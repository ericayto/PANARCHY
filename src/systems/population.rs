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
            let (labor_demand, matching_efficiency, food_shortage_ratio) = world
                .economies
                .get(&id)
                .map(|econ| {
                    (
                        econ.labor_demand,
                        econ.job_matching_efficiency,
                        econ.food_shortage_ratio,
                    )
                })
                .unwrap_or_else(|| {
                    (
                        population.citizens as f64 * population.target_employment_rate,
                        1.0,
                        0.0,
                    )
                });
            let dt_years = ctx.dt_days / 365.0;
            let births = (population.citizens as f64 * population.annual_birth_rate * dt_years)
                .round() as i64;
            let deaths = (population.citizens as f64 * population.annual_death_rate * dt_years)
                .round() as i64;
            let mut net_delta = births - deaths;

            let starvation_penalty =
                (population.citizens as f64 * food_shortage_ratio * 0.05).ceil() as i64;
            if starvation_penalty > 0 {
                net_delta -= starvation_penalty;
                world.bookkeeping.starving_regions.push(region_name.clone());
            }

            let shock: f64 = rng.gen_range(0.975..1.025);
            let desired_employment = (labor_demand * matching_efficiency * shock).round() as i64;
            let employed = desired_employment
                .clamp(0, population.citizens as i64)
                .max(0) as u64;

            let next_citizens = (population.citizens as i64 + net_delta).max(0) as u64;
            population.citizens = next_citizens;
            population.employed = employed.min(population.citizens);
        }
        Ok(())
    }
}
