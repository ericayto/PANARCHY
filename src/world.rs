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
    pub energy_consumption_per_capita: f64,
    pub target_employment_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomyComponent {
    pub food_productivity_per_worker: f64,
    pub energy_productivity_per_worker: f64,
    pub wage: f64,
    pub target_inventory_days: f64,
    pub price_adjustment_rate: f64,
    pub wage_adjustment_rate: f64,
    pub job_matching_efficiency: f64,
    pub basic_income_per_capita: f64,
    pub propensity_to_consume: f64,
    pub food_price: f64,
    pub energy_price: f64,
    pub labor_demand: f64,
    pub household_budget: f64,
    pub food_shortage_ratio: f64,
    pub energy_shortage_ratio: f64,
    pub wage_bill: f64,
    pub sales_revenue: f64,
    pub energy_dispatched: f64,
    pub energy_curtailed: f64,
    pub transport_utilization: f64,
    pub transport_shortfall: f64,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinanceComponent {
    pub bank_deposits: f64,
    pub loan_balance: f64,
    pub policy_rate: f64,
    pub loan_rate_spread: f64,
    pub deposit_rate: f64,
    pub default_rate: f64,
    pub target_loan_to_deposit: f64,
    pub infrastructure_spend_fraction: f64,
    pub credit_stress: f64,
    pub cumulative_defaults: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfrastructureComponent {
    pub power_capacity: f64,
    pub transport_capacity: f64,
    pub maintenance_cost: f64,
    pub degradation_rate: f64,
    pub reliability: f64,
    pub pending_investment: f64,
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
    pub unemployment_rate: f64,
    pub food: f64,
    pub energy: f64,
    pub wage: f64,
    pub labor_demand: f64,
    pub household_budget: f64,
    pub food_price: f64,
    pub energy_price: f64,
    pub food_shortage_ratio: f64,
    pub energy_shortage_ratio: f64,
    pub bank_deposits: f64,
    pub loan_balance: f64,
    pub credit_stress: f64,
    pub power_capacity: f64,
    pub transport_capacity: f64,
    pub infrastructure_reliability: f64,
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
    pub(crate) economies: HashMap<EntityId, EconomyComponent>,
    pub(crate) resources: HashMap<EntityId, ResourceStock>,
    pub(crate) finances: HashMap<EntityId, FinanceComponent>,
    pub(crate) infrastructure: HashMap<EntityId, InfrastructureComponent>,
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
            economies: HashMap::new(),
            resources: HashMap::new(),
            finances: HashMap::new(),
            infrastructure: HashMap::new(),
            bookkeeping: BookkeepingState::default(),
        }
    }

    pub fn spawn_region(
        &mut self,
        region: RegionComponent,
        population: PopulationComponent,
        economy: EconomyComponent,
        resources: ResourceStock,
        finance: FinanceComponent,
        infrastructure: InfrastructureComponent,
    ) -> EntityId {
        let id = self.allocate();
        self.regions.insert(id, region);
        self.populations.insert(id, population);
        self.economies.insert(id, economy);
        self.resources.insert(id, resources);
        self.finances.insert(id, finance);
        self.infrastructure.insert(id, infrastructure);
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
            let economy = self.economies.get(id).expect("economy component exists");
            let stock = self.resources.get(id).expect("resource component exists");
            let finance = self.finances.get(id);
            let infra = self.infrastructure.get(id);
            let unemployment_rate = if population.citizens > 0 {
                1.0 - (population.employed as f64 / population.citizens as f64)
            } else {
                0.0
            };
            regions.push(RegionSnapshot {
                id: id.raw(),
                name: region.name.clone(),
                citizens: population.citizens,
                employed: population.employed,
                unemployment_rate,
                food: stock.food,
                energy: stock.energy,
                wage: economy.wage,
                labor_demand: economy.labor_demand,
                household_budget: economy.household_budget,
                food_price: economy.food_price,
                energy_price: economy.energy_price,
                food_shortage_ratio: economy.food_shortage_ratio,
                energy_shortage_ratio: economy.energy_shortage_ratio,
                bank_deposits: finance.map(|f| f.bank_deposits).unwrap_or(0.0),
                loan_balance: finance.map(|f| f.loan_balance).unwrap_or(0.0),
                credit_stress: finance.map(|f| f.credit_stress).unwrap_or(0.0),
                power_capacity: infra.map(|i| i.power_capacity).unwrap_or(0.0),
                transport_capacity: infra.map(|i| i.transport_capacity).unwrap_or(0.0),
                infrastructure_reliability: infra.map(|i| i.reliability).unwrap_or(0.0),
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

    pub fn entity_ids(&self) -> Vec<EntityId> {
        let mut ids: Vec<_> = self.regions.keys().cloned().collect();
        ids.sort();
        ids
    }

    pub fn economy(&self, id: EntityId) -> Option<&EconomyComponent> {
        self.economies.get(&id)
    }

    pub fn economy_mut(&mut self, id: EntityId) -> Option<&mut EconomyComponent> {
        self.economies.get_mut(&id)
    }

    pub fn resources_mut(&mut self, id: EntityId) -> Option<&mut ResourceStock> {
        self.resources.get_mut(&id)
    }

    pub fn population(&self, id: EntityId) -> Option<&PopulationComponent> {
        self.populations.get(&id)
    }

    pub fn finance(&self, id: EntityId) -> Option<&FinanceComponent> {
        self.finances.get(&id)
    }

    pub fn finance_mut(&mut self, id: EntityId) -> Option<&mut FinanceComponent> {
        self.finances.get_mut(&id)
    }

    pub fn infrastructure(&self, id: EntityId) -> Option<&InfrastructureComponent> {
        self.infrastructure.get(&id)
    }

    pub fn infrastructure_mut(&mut self, id: EntityId) -> Option<&mut InfrastructureComponent> {
        self.infrastructure.get_mut(&id)
    }
    fn allocate(&mut self) -> EntityId {
        let id = EntityId(self.next_entity);
        self.next_entity += 1;
        id
    }
}
