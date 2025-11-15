use anyhow::Result;

use crate::{
    engine::{System, SystemContext},
    rng::SystemRng,
    world::{EntityId, World},
};

const EPS: f64 = 1e-9;

pub struct EconomySystem;

impl EconomySystem {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EconomySystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for EconomySystem {
    fn name(&self) -> &str {
        "economy"
    }

    fn run(
        &mut self,
        ctx: &SystemContext,
        world: &mut World,
        _rng: &mut SystemRng<'_>,
    ) -> Result<()> {
        let dt = ctx.dt_days;
        let mut ids: Vec<EntityId> = world.economies.keys().cloned().collect();
        ids.sort();
        for id in ids {
            let (citizens, employed, food_per_capita, energy_per_capita) =
                match world.populations.get(&id) {
                    Some(pop) => (
                        pop.citizens as f64,
                        pop.employed as f64,
                        pop.food_consumption_per_capita,
                        pop.energy_consumption_per_capita,
                    ),
                    None => continue,
                };
            let stock = match world.resources.get_mut(&id) {
                Some(stock) => stock,
                None => continue,
            };
            let economy = match world.economies.get_mut(&id) {
                Some(economy) => economy,
                None => continue,
            };
            let (power_capacity, transport_capacity) = world
                .infrastructure
                .get(&id)
                .map(|infra| (infra.power_capacity, infra.transport_capacity))
                .unwrap_or((f64::INFINITY, f64::INFINITY));

            if citizens <= 0.0 {
                economy.labor_demand = 0.0;
                economy.household_budget = 0.0;
                economy.food_shortage_ratio = 0.0;
                economy.energy_shortage_ratio = 0.0;
                economy.energy_dispatched = 0.0;
                economy.energy_curtailed = 0.0;
                economy.transport_utilization = 0.0;
                economy.transport_shortfall = 0.0;
                economy.wage_bill = 0.0;
                economy.sales_revenue = 0.0;
                continue;
            }

            let desired_food = citizens * food_per_capita * dt;
            let desired_energy = citizens * energy_per_capita * dt;

            let inventory_target_food = desired_food * economy.target_inventory_days;
            let inventory_target_energy = desired_energy * economy.target_inventory_days;
            let food_gap = (inventory_target_food - stock.food).max(0.0);
            let energy_gap = (inventory_target_energy - stock.energy).max(0.0);

            let per_worker_food = (economy.food_productivity_per_worker * dt).max(EPS);
            let per_worker_energy = (economy.energy_productivity_per_worker * dt).max(EPS);

            let labor_needed_food = (desired_food + food_gap) / per_worker_food;
            let labor_needed_energy = (desired_energy + energy_gap) / per_worker_energy;
            let total_labor_needed = labor_needed_food + labor_needed_energy;
            economy.labor_demand = total_labor_needed.max(0.0);

            let (food_workers, energy_workers) = if total_labor_needed > EPS {
                (
                    employed * (labor_needed_food / total_labor_needed),
                    employed * (labor_needed_energy / total_labor_needed),
                )
            } else {
                (employed * 0.5, employed * 0.5)
            };

            stock.food += food_workers * per_worker_food;
            let energy_output = energy_workers * per_worker_energy;
            let max_energy_dispatch = if power_capacity.is_finite() {
                (power_capacity * dt).max(0.0)
            } else {
                f64::INFINITY
            };
            let energy_dispatched = if max_energy_dispatch.is_finite() {
                energy_output.min(max_energy_dispatch)
            } else {
                energy_output
            };
            let curtailed_energy = (energy_output - energy_dispatched).max(0.0);
            stock.energy += energy_dispatched;
            economy.energy_dispatched = energy_dispatched;
            economy.energy_curtailed = curtailed_energy;

            let wage_income = economy.wage * employed * dt;
            let unemployed = (citizens - employed).max(0.0);
            let basic_income = economy.basic_income_per_capita * unemployed * dt;
            let mut budget = (wage_income + basic_income).max(0.0);
            budget *= economy.propensity_to_consume;
            economy.household_budget = budget;
            economy.wage_bill = wage_income;

            let desired_cost =
                desired_food * economy.food_price + desired_energy * economy.energy_price;
            let demand_scale = if desired_cost > EPS {
                (budget / desired_cost).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let scaled_food_demand = desired_food * demand_scale;
            let scaled_energy_demand = desired_energy * demand_scale;

            let transport_capacity_per_tick = if transport_capacity.is_finite() {
                (transport_capacity * dt).max(0.0)
            } else {
                f64::INFINITY
            };
            let total_dispatch = scaled_food_demand + scaled_energy_demand;
            let transport_ratio = if transport_capacity_per_tick.is_finite()
                && transport_capacity_per_tick < total_dispatch - EPS
                && total_dispatch > EPS
            {
                (transport_capacity_per_tick / total_dispatch).clamp(0.0, 1.0)
            } else {
                1.0
            };
            let deliverable_food = scaled_food_demand * transport_ratio;
            let deliverable_energy = scaled_energy_demand * transport_ratio;

            let sold_food = deliverable_food.min(stock.food);
            stock.food -= sold_food;
            let sold_energy = deliverable_energy.min(stock.energy);
            stock.energy -= sold_energy;
            let delivered_total = sold_food + sold_energy;
            let transport_utilization =
                if transport_capacity_per_tick.is_finite() && transport_capacity_per_tick > EPS {
                    (delivered_total / transport_capacity_per_tick).clamp(0.0, 1.0)
                } else {
                    0.0
                };
            let transport_shortfall = if total_dispatch > EPS {
                ((total_dispatch - delivered_total).max(0.0) / total_dispatch).clamp(0.0, 1.0)
            } else {
                0.0
            };
            economy.transport_utilization = transport_utilization;
            economy.transport_shortfall = transport_shortfall;

            let food_shortage_ratio = if desired_food > EPS {
                ((desired_food - sold_food).max(0.0) / desired_food).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let energy_shortage_ratio = if desired_energy > EPS {
                ((desired_energy - sold_energy).max(0.0) / desired_energy).clamp(0.0, 1.0)
            } else {
                0.0
            };
            economy.food_shortage_ratio = food_shortage_ratio;
            economy.energy_shortage_ratio = energy_shortage_ratio;
            economy.sales_revenue =
                sold_food * economy.food_price + sold_energy * economy.energy_price;

            adjust_price(
                &mut economy.food_price,
                food_shortage_ratio,
                stock.food,
                inventory_target_food,
                economy.price_adjustment_rate,
            );
            adjust_price(
                &mut economy.energy_price,
                energy_shortage_ratio,
                stock.energy,
                inventory_target_energy,
                economy.price_adjustment_rate,
            );

            adjust_wages(
                &mut economy.wage,
                economy.wage_adjustment_rate,
                economy.labor_demand,
                employed,
                citizens,
            );
        }
        Ok(())
    }
}

fn adjust_price(
    price: &mut f64,
    shortage_ratio: f64,
    inventory: f64,
    target_inventory: f64,
    adjustment_rate: f64,
) {
    let mut next_price = *price;
    if shortage_ratio > 0.001 {
        let pressure = shortage_ratio.min(1.0);
        next_price *= 1.0 + adjustment_rate * pressure;
    } else {
        let ratio = if target_inventory > EPS {
            inventory / target_inventory
        } else {
            1.0
        };
        if ratio > 1.15 {
            let drop = ((ratio - 1.0) / ratio).min(0.5);
            next_price *= 1.0 - adjustment_rate * drop;
        }
    }
    *price = next_price.max(0.1);
}

fn adjust_wages(wage: &mut f64, rate: f64, labor_demand: f64, employed: f64, citizens: f64) {
    if citizens <= 0.0 {
        return;
    }
    let gap = labor_demand - employed;
    let gap_ratio = (gap / citizens).clamp(-0.5, 0.5);
    let mut next = *wage * (1.0 + rate * gap_ratio);
    if !next.is_finite() {
        next = *wage;
    }
    *wage = next.max(1.0);
}
