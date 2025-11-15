use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EntityId(u64);

impl EntityId {
    pub fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionComponent {
    pub name: String,
    pub food_regen_per_1000: f64,
    pub energy_regen_per_1000: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationComponent {
    pub citizens: u64,
    pub employed: u64,
    pub annual_birth_rate: f64,
    pub annual_death_rate: f64,
    pub food_consumption_per_capita: f64,
    pub target_employment_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStock {
    pub food: f64,
    pub energy: f64,
}

impl ResourceStock {
    pub fn clamp_non_negative(&mut self) {
        self.food = self.food.max(0.0);
        self.energy = self.energy.max(0.0);
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BookkeepingState {
    pub starving_regions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegionSnapshot {
    pub id: u64,
    pub name: String,
    pub citizens: u64,
    pub employed: u64,
    pub food: f64,
    pub energy: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub scenario: String,
    pub tick: u64,
    pub days_elapsed: f64,
    pub total_population: u64,
    pub starving_regions: Vec<String>,
    pub regions: Vec<RegionSnapshot>,
}

pub struct World {
    next_entity: u64,
    tick: u64,
    days_elapsed: f64,
    dt_days: f64,
    pub(crate) regions: HashMap<EntityId, RegionComponent>,
    pub(crate) populations: HashMap<EntityId, PopulationComponent>,
    pub(crate) resources: HashMap<EntityId, ResourceStock>,
    pub(crate) bookkeeping: BookkeepingState,
}

impl World {
    pub fn new(dt_days: f64) -> Self {
        Self {
            next_entity: 0,
            tick: 0,
            days_elapsed: 0.0,
            dt_days,
            regions: HashMap::new(),
            populations: HashMap::new(),
            resources: HashMap::new(),
            bookkeeping: BookkeepingState::default(),
        }
    }

    pub fn spawn_region(
        &mut self,
        region: RegionComponent,
        population: PopulationComponent,
        resources: ResourceStock,
    ) -> EntityId {
        let id = self.allocate();
        self.regions.insert(id, region);
        self.populations.insert(id, population);
        self.resources.insert(id, resources);
        id
    }

    pub fn tick(&self) -> u64 {
        self.tick
    }

    pub fn dt_days(&self) -> f64 {
        self.dt_days
    }

    pub fn advance_time(&mut self) {
        self.tick += 1;
        self.days_elapsed += self.dt_days;
    }

    pub fn days_elapsed(&self) -> f64 {
        self.days_elapsed
    }

    pub fn total_population(&self) -> u64 {
        self.populations.values().map(|p| p.citizens).sum()
    }

    pub fn snapshot(&self, scenario: &str) -> WorldSnapshot {
        let mut regions: Vec<RegionSnapshot> = Vec::with_capacity(self.regions.len());
        for (id, region) in &self.regions {
            let population = self
                .populations
                .get(id)
                .expect("population component exists");
            let stock = self.resources.get(id).expect("resource component exists");
            regions.push(RegionSnapshot {
                id: id.raw(),
                name: region.name.clone(),
                citizens: population.citizens,
                employed: population.employed,
                food: stock.food,
                energy: stock.energy,
            });
        }
        regions.sort_by_key(|r| r.id);
        WorldSnapshot {
            scenario: scenario.to_string(),
            tick: self.tick,
            days_elapsed: self.days_elapsed,
            total_population: self.total_population(),
            starving_regions: self.bookkeeping.starving_regions.clone(),
            regions,
        }
    }

    fn allocate(&mut self) -> EntityId {
        let id = EntityId(self.next_entity);
        self.next_entity += 1;
        id
    }
}
