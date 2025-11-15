use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::world::{
    EconomyComponent, FinanceComponent, InfrastructureComponent, PolicyComponent,
    PopulationComponent, RegionComponent, ResourceStock, TechnologyComponent, World,
};

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

fn default_energy_consumption() -> f64 {
    1.3
}

fn default_food_regen() -> f64 {
    55.0
}

fn default_energy_regen() -> f64 {
    20.0
}

fn default_food_productivity() -> f64 {
    3.2
}

fn default_energy_productivity() -> f64 {
    4.2
}

fn default_wage_per_worker() -> f64 {
    120.0
}

fn default_food_price() -> f64 {
    2.0
}

fn default_energy_price() -> f64 {
    1.25
}

fn default_inventory_days() -> f64 {
    20.0
}

fn default_price_adjustment() -> f64 {
    0.04
}

fn default_wage_adjustment() -> f64 {
    0.02
}

fn default_job_matching_efficiency() -> f64 {
    0.92
}

fn default_basic_income() -> f64 {
    15.0
}

fn default_propensity_to_consume() -> f64 {
    0.9
}

fn default_initial_deposits() -> f64 {
    5_000_000.0
}

fn default_initial_loans() -> f64 {
    0.0
}

fn default_policy_rate() -> f64 {
    0.02
}

fn default_loan_rate_spread() -> f64 {
    0.02
}

fn default_deposit_rate() -> f64 {
    0.01
}

fn default_default_rate() -> f64 {
    0.01
}

fn default_target_loan_to_deposit() -> f64 {
    0.9
}

fn default_infrastructure_spend_fraction() -> f64 {
    0.12
}

fn default_power_capacity() -> f64 {
    65_000.0
}

fn default_transport_capacity() -> f64 {
    75_000.0
}

fn default_infrastructure_maintenance_cost() -> f64 {
    12_000.0
}

fn default_infrastructure_degradation_rate() -> f64 {
    0.003
}

fn default_tax_rate() -> f64 {
    0.24
}

fn default_public_investment_fraction() -> f64 {
    0.18
}

fn default_rnd_fraction() -> f64 {
    0.12
}

fn default_target_unemployment() -> f64 {
    0.07
}

fn default_target_primary_balance() -> f64 {
    0.0
}

fn default_rnd_budget_per_capita() -> f64 {
    8.0
}

fn default_research_efficiency_param() -> f64 {
    1.0
}

fn default_starting_techs() -> Vec<String> {
    Vec::new()
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
    #[serde(default = "default_energy_consumption")]
    pub energy_consumption_per_capita: f64,
    pub resources: ResourceInit,
    #[serde(default)]
    pub regen: ResourceRegen,
    #[serde(default)]
    pub economy: ScenarioEconomy,
    #[serde(default)]
    pub finance: ScenarioFinance,
    #[serde(default)]
    pub infrastructure: ScenarioInfrastructure,
    #[serde(default)]
    pub technology: ScenarioTechnology,
    #[serde(default)]
    pub policy: ScenarioPolicy,
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

#[derive(Debug, Clone, Deserialize)]
pub struct ScenarioEconomy {
    #[serde(default = "default_food_productivity")]
    pub food_productivity_per_worker: f64,
    #[serde(default = "default_energy_productivity")]
    pub energy_productivity_per_worker: f64,
    #[serde(default = "default_wage_per_worker")]
    pub wage_per_worker: f64,
    #[serde(default = "default_inventory_days")]
    pub target_inventory_days: f64,
    #[serde(default = "default_price_adjustment")]
    pub price_adjustment_rate: f64,
    #[serde(default = "default_wage_adjustment")]
    pub wage_adjustment_rate: f64,
    #[serde(default = "default_food_price")]
    pub food_price: f64,
    #[serde(default = "default_energy_price")]
    pub energy_price: f64,
    #[serde(default = "default_job_matching_efficiency")]
    pub job_matching_efficiency: f64,
    #[serde(default = "default_basic_income")]
    pub basic_income_per_capita: f64,
    #[serde(default = "default_propensity_to_consume")]
    pub propensity_to_consume: f64,
}

impl Default for ScenarioEconomy {
    fn default() -> Self {
        Self {
            food_productivity_per_worker: default_food_productivity(),
            energy_productivity_per_worker: default_energy_productivity(),
            wage_per_worker: default_wage_per_worker(),
            target_inventory_days: default_inventory_days(),
            price_adjustment_rate: default_price_adjustment(),
            wage_adjustment_rate: default_wage_adjustment(),
            food_price: default_food_price(),
            energy_price: default_energy_price(),
            job_matching_efficiency: default_job_matching_efficiency(),
            basic_income_per_capita: default_basic_income(),
            propensity_to_consume: default_propensity_to_consume(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScenarioFinance {
    #[serde(default = "default_initial_deposits")]
    pub initial_deposits: f64,
    #[serde(default = "default_initial_loans")]
    pub initial_loans: f64,
    #[serde(default = "default_policy_rate")]
    pub policy_rate: f64,
    #[serde(default = "default_loan_rate_spread")]
    pub loan_rate_spread: f64,
    #[serde(default = "default_deposit_rate")]
    pub deposit_rate: f64,
    #[serde(default = "default_default_rate")]
    pub default_rate: f64,
    #[serde(default = "default_target_loan_to_deposit")]
    pub target_loan_to_deposit: f64,
    #[serde(default = "default_infrastructure_spend_fraction")]
    pub infrastructure_spend_fraction: f64,
}

impl Default for ScenarioFinance {
    fn default() -> Self {
        Self {
            initial_deposits: default_initial_deposits(),
            initial_loans: default_initial_loans(),
            policy_rate: default_policy_rate(),
            loan_rate_spread: default_loan_rate_spread(),
            deposit_rate: default_deposit_rate(),
            default_rate: default_default_rate(),
            target_loan_to_deposit: default_target_loan_to_deposit(),
            infrastructure_spend_fraction: default_infrastructure_spend_fraction(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScenarioInfrastructure {
    #[serde(default = "default_power_capacity")]
    pub power_capacity: f64,
    #[serde(default = "default_transport_capacity")]
    pub transport_capacity: f64,
    #[serde(default = "default_infrastructure_maintenance_cost")]
    pub maintenance_cost: f64,
    #[serde(default = "default_infrastructure_degradation_rate")]
    pub degradation_rate: f64,
}

impl Default for ScenarioInfrastructure {
    fn default() -> Self {
        Self {
            power_capacity: default_power_capacity(),
            transport_capacity: default_transport_capacity(),
            maintenance_cost: default_infrastructure_maintenance_cost(),
            degradation_rate: default_infrastructure_degradation_rate(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScenarioTechnology {
    #[serde(default = "default_rnd_budget_per_capita")]
    pub rnd_budget_per_capita: f64,
    #[serde(default = "default_research_efficiency_param")]
    pub research_efficiency: f64,
    #[serde(default = "default_starting_techs")]
    pub starting_techs: Vec<String>,
}

impl Default for ScenarioTechnology {
    fn default() -> Self {
        Self {
            rnd_budget_per_capita: default_rnd_budget_per_capita(),
            research_efficiency: default_research_efficiency_param(),
            starting_techs: default_starting_techs(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScenarioPolicy {
    #[serde(default = "default_tax_rate")]
    pub tax_rate: f64,
    #[serde(default)]
    pub transfer_per_capita: Option<f64>,
    #[serde(default = "default_public_investment_fraction")]
    pub public_investment_fraction: f64,
    #[serde(default = "default_rnd_fraction")]
    pub rnd_fraction: f64,
    #[serde(default = "default_target_unemployment")]
    pub target_unemployment_rate: f64,
    #[serde(default = "default_target_primary_balance")]
    pub target_primary_balance: f64,
}

impl Default for ScenarioPolicy {
    fn default() -> Self {
        Self {
            tax_rate: default_tax_rate(),
            transfer_per_capita: None,
            public_investment_fraction: default_public_investment_fraction(),
            rnd_fraction: default_rnd_fraction(),
            target_unemployment_rate: default_target_unemployment(),
            target_primary_balance: default_target_primary_balance(),
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
            let transfer_per_capita = region
                .policy
                .transfer_per_capita
                .unwrap_or(region.economy.basic_income_per_capita);
            let population = PopulationComponent {
                citizens: region.citizens,
                employed,
                annual_birth_rate: region.annual_birth_rate,
                annual_death_rate: region.annual_death_rate,
                food_consumption_per_capita: region.food_consumption_per_capita,
                energy_consumption_per_capita: region.energy_consumption_per_capita,
                target_employment_rate: region.employment_rate,
            };
            let region_component = RegionComponent {
                name: region.name.clone(),
                food_regen_per_1000: region.regen.food_per_1000,
                energy_regen_per_1000: region.regen.energy_per_1000,
            };
            let (food_multiplier, energy_multiplier) =
                crate::technology::aggregate_productivity_multipliers(
                    &region.technology.starting_techs,
                );
            let economy = EconomyComponent {
                food_productivity_per_worker: region.economy.food_productivity_per_worker
                    * food_multiplier,
                energy_productivity_per_worker: region.economy.energy_productivity_per_worker
                    * energy_multiplier,
                wage: region.economy.wage_per_worker,
                target_inventory_days: region.economy.target_inventory_days,
                price_adjustment_rate: region.economy.price_adjustment_rate,
                wage_adjustment_rate: region.economy.wage_adjustment_rate,
                job_matching_efficiency: region.economy.job_matching_efficiency,
                basic_income_per_capita: transfer_per_capita,
                propensity_to_consume: region.economy.propensity_to_consume,
                food_price: region.economy.food_price,
                energy_price: region.economy.energy_price,
                labor_demand: employed as f64,
                household_budget: 0.0,
                food_shortage_ratio: 0.0,
                energy_shortage_ratio: 0.0,
                wage_bill: 0.0,
                sales_revenue: 0.0,
                energy_dispatched: 0.0,
                energy_curtailed: 0.0,
                transport_utilization: 0.0,
                transport_shortfall: 0.0,
            };
            let stock = ResourceStock {
                food: region.resources.food,
                energy: region.resources.energy,
            };
            let finance = FinanceComponent {
                bank_deposits: region.finance.initial_deposits,
                loan_balance: region.finance.initial_loans,
                policy_rate: region.finance.policy_rate,
                loan_rate_spread: region.finance.loan_rate_spread,
                deposit_rate: region.finance.deposit_rate,
                default_rate: region.finance.default_rate,
                target_loan_to_deposit: region.finance.target_loan_to_deposit,
                infrastructure_spend_fraction: region.finance.infrastructure_spend_fraction,
                credit_stress: 0.0,
                cumulative_defaults: 0.0,
            };
            let infrastructure = InfrastructureComponent {
                power_capacity: region.infrastructure.power_capacity,
                transport_capacity: region.infrastructure.transport_capacity,
                maintenance_cost: region.infrastructure.maintenance_cost,
                degradation_rate: region.infrastructure.degradation_rate,
                reliability: 1.0,
                pending_investment: 0.0,
            };
            let technology = TechnologyComponent {
                base_food_productivity: region.economy.food_productivity_per_worker,
                base_energy_productivity: region.economy.energy_productivity_per_worker,
                unlocked: region.technology.starting_techs.clone(),
                active_project: None,
                research_efficiency: region.technology.research_efficiency,
                baseline_rnd_budget_per_capita: region.technology.rnd_budget_per_capita,
                current_allocation: 0.0,
                innovation_score: 0.0,
            };
            let policy = PolicyComponent {
                tax_rate: region.policy.tax_rate,
                transfer_per_capita,
                public_investment_fraction: region.policy.public_investment_fraction,
                rnd_fraction: region.policy.rnd_fraction,
                target_unemployment_rate: region.policy.target_unemployment_rate,
                target_primary_balance: region.policy.target_primary_balance,
                budget_balance: 0.0,
                public_debt: 0.0,
                approval_rating: 0.65,
                last_tax_revenue: 0.0,
                last_transfers: 0.0,
                last_public_investment: 0.0,
                last_rnd_allocation: 0.0,
            };
            world.spawn_region(
                region_component,
                population,
                economy,
                stock,
                finance,
                infrastructure,
                technology,
                policy,
            );
        }
        world
    }

    pub fn ticks(&self, override_ticks: Option<u64>) -> u64 {
        override_ticks.or(self.ticks).unwrap_or(120)
    }
}
