use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::world::{PopulationComponent, RegionComponent, ResourceStock, World};

fn default_dt_days() -> f64 {
    1.0
}

fn default_snapshot_interval_ticks() -> u64 {
    30
}

fn default_employment_rate() -> f64 {
    0.65
}

fn default_birth_rate() -> f64 {
    0.011
}

fn default_death_rate() -> f64 {
    0.008
}

fn default_food_consumption() -> f64 {
    1.7
}

fn default_food_regen() -> f64 {
    55.0
}

fn default_energy_regen() -> f64 {
    20.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct Scenario {
    pub name: String,
    pub description: Option<String>,
    pub seed: u64,
    #[serde(default = "default_dt_days")]
    pub dt_days: f64,
    #[serde(default)]
    pub ticks: Option<u64>,
    #[serde(default = "default_snapshot_interval_ticks")]
    pub snapshot_interval_ticks: u64,
    pub regions: Vec<ScenarioRegion>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScenarioRegion {
    pub name: String,
    pub citizens: u64,
    #[serde(default = "default_employment_rate")]
    pub employment_rate: f64,
    #[serde(default = "default_birth_rate")]
    pub annual_birth_rate: f64,
    #[serde(default = "default_death_rate")]
    pub annual_death_rate: f64,
    #[serde(default = "default_food_consumption")]
    pub food_consumption_per_capita: f64,
    pub resources: ResourceInit,
    #[serde(default)]
    pub regen: ResourceRegen,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResourceInit {
    pub food: f64,
    pub energy: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResourceRegen {
    #[serde(default = "default_food_regen")]
    pub food_per_1000: f64,
    #[serde(default = "default_energy_regen")]
    pub energy_per_1000: f64,
}

impl Default for ResourceRegen {
    fn default() -> Self {
        Self {
            food_per_1000: default_food_regen(),
            energy_per_1000: default_energy_regen(),
        }
    }
}

pub struct ScenarioLoader {
    base_dir: PathBuf,
}

impl ScenarioLoader {
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
        }
    }

    pub fn load(&self, file: impl AsRef<Path>) -> Result<Scenario> {
        let path = self.base_dir.join(file);
        let data = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read scenario file {}", path.display()))?;
        let scenario: Scenario = serde_yaml::from_str(&data)
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        Ok(scenario)
    }
}

impl Scenario {
    pub fn build_world(&self) -> World {
        let mut world = World::new(self.dt_days);
        for region in &self.regions {
            let employed = (region.citizens as f64 * region.employment_rate)
                .round()
                .clamp(0.0, region.citizens as f64) as u64;
            let population = PopulationComponent {
                citizens: region.citizens,
                employed,
                annual_birth_rate: region.annual_birth_rate,
                annual_death_rate: region.annual_death_rate,
                food_consumption_per_capita: region.food_consumption_per_capita,
                target_employment_rate: region.employment_rate,
            };
            let region_component = RegionComponent {
                name: region.name.clone(),
                food_regen_per_1000: region.regen.food_per_1000,
                energy_regen_per_1000: region.regen.energy_per_1000,
            };
            let stock = ResourceStock {
                food: region.resources.food,
                energy: region.resources.energy,
            };
            world.spawn_region(region_component, population, stock);
        }
        world
    }

    pub fn ticks(&self, override_ticks: Option<u64>) -> u64 {
        override_ticks.or(self.ticks).unwrap_or(120)
    }
}
