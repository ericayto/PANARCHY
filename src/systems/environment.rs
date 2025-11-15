use crate::engine::DeterministicRng;
use crate::world::WorldState;

use super::System;

#[derive(Default)]
pub struct EnvironmentSystem;

impl System for EnvironmentSystem {
    fn name(&self) -> &'static str {
        "environment"
    }

    fn update(
        &mut self,
        world: &mut WorldState,
        rng: &mut DeterministicRng,
        tick: u64,
        dt_days: f64,
    ) {
        let dt_years = (dt_days / 365.0) as f32;
        for tile in &mut world.tiles {
            tile.environment.seasonal_phase = (tile.environment.seasonal_phase + dt_years).fract();
            let seasonal = (tile.environment.seasonal_phase * std::f32::consts::TAU).sin();
            let noise = rng.gen_range(-0.05, 0.05);
            tile.environment.temperature_c += 0.2 * seasonal + noise;
            tile.environment.precipitation_mm =
                (tile.environment.precipitation_mm + 0.05 * seasonal).max(0.0);
            tile.environment.fertility =
                (tile.environment.fertility + 0.01 * seasonal).clamp(0.2, 1.2);
        }

        let _ = tick;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
