use anyhow::Result;

use crate::{
    engine::{System, SystemContext},
    rng::SystemRng,
    world::{EntityId, World},
};

pub struct PolicySystem;

impl PolicySystem {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PolicySystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for PolicySystem {
    fn name(&self) -> &str {
        "policy"
    }

    fn run(
        &mut self,
        ctx: &SystemContext,
        world: &mut World,
        _rng: &mut SystemRng<'_>,
    ) -> Result<()> {
        let dt = ctx.dt_days;
        let mut ids: Vec<EntityId> = world.policies.keys().cloned().collect();
        ids.sort();
        for id in ids {
            let population = match world.populations.get(&id) {
                Some(pop) => (pop.citizens as f64, pop.employed as f64),
                None => continue,
            };
            if population.0 <= 0.0 {
                continue;
            }
            let economy_snapshot = match world.economies.get(&id) {
                Some(econ) => (
                    econ.sales_revenue,
                    econ.food_shortage_ratio,
                    econ.energy_shortage_ratio,
                    econ.transport_shortfall,
                ),
                None => continue,
            };
            let baseline_rnd = world
                .technology
                .get(&id)
                .map(|tech| tech.baseline_rnd_budget_per_capita)
                .unwrap_or(0.0);
            let citizens = population.0;
            let employed = population.1;
            let unemployment_rate = if citizens > 0.0 {
                1.0 - (employed / citizens)
            } else {
                0.0
            };
            let (sales_revenue, food_shortage, energy_shortage, transport_shortfall) =
                economy_snapshot;
            let (rnd_allocation, public_investment, updated_transfer) = {
                let policy = match world.policies.get_mut(&id) {
                    Some(policy) => policy,
                    None => continue,
                };
                let gdp = sales_revenue.max(0.0);
                let tax_revenue = (gdp * policy.tax_rate.max(0.0)).max(0.0);
                policy.last_tax_revenue = tax_revenue;
                let unemployed = (citizens - employed).max(0.0);
                let transfers = policy.transfer_per_capita.max(0.0) * unemployed * dt;
                policy.last_transfers = transfers;
                let discretionary = tax_revenue - transfers;
                let guaranteed_rnd = citizens * baseline_rnd.max(0.0) * dt;
                let extra_rnd = discretionary.max(0.0) * policy.rnd_fraction.max(0.0);
                let rnd_allocation = (guaranteed_rnd + extra_rnd).max(0.0);
                let remaining = discretionary - extra_rnd;
                let public_investment =
                    remaining.max(0.0) * policy.public_investment_fraction.max(0.0);
                policy.last_public_investment = public_investment;
                policy.last_rnd_allocation = rnd_allocation;

                let spending = transfers + public_investment + rnd_allocation;
                policy.budget_balance = tax_revenue - spending;
                if policy.budget_balance < 0.0 {
                    policy.public_debt += -policy.budget_balance;
                } else {
                    policy.public_debt =
                        (policy.public_debt - policy.budget_balance * 0.35).max(0.0);
                }

                let unemployment_gap = unemployment_rate - policy.target_unemployment_rate;
                if unemployment_gap > 0.01 {
                    let pressure = unemployment_gap.min(0.25);
                    policy.transfer_per_capita *= 1.0 + 0.5 * pressure;
                    policy.tax_rate *= 1.0 - 0.06 * pressure;
                } else if unemployment_gap < -0.01 {
                    let pressure = (-unemployment_gap).min(0.25);
                    policy.transfer_per_capita *= 1.0 - 0.4 * pressure;
                    policy.tax_rate *= 1.0 + 0.05 * pressure;
                }

                let balance_gap = policy.budget_balance - policy.target_primary_balance;
                if balance_gap < 0.0 {
                    let severity = (-balance_gap / (tax_revenue.abs() + 1.0)).clamp(0.0, 0.1);
                    policy.tax_rate *= 1.0 + severity;
                } else {
                    policy.tax_rate *= 0.999;
                }

                let shortage_signal =
                    (food_shortage + energy_shortage) * 0.5 + transport_shortfall * 0.5;
                let approval_signal = (1.0 - unemployment_rate).clamp(0.0, 1.0) * 0.6
                    + (1.0 - shortage_signal).clamp(0.0, 1.0) * 0.4;
                policy.approval_rating =
                    (policy.approval_rating * 0.85 + approval_signal * 0.15).clamp(0.0, 1.0);

                policy.tax_rate = policy.tax_rate.clamp(0.04, 0.65);
                policy.transfer_per_capita = policy.transfer_per_capita.clamp(5.0, 400.0);
                (
                    rnd_allocation,
                    public_investment,
                    policy.transfer_per_capita,
                )
            };

            if let Some(econ) = world.economies.get_mut(&id) {
                econ.basic_income_per_capita = updated_transfer;
            }

            if public_investment > 0.0 {
                if let Some(infra) = world.infrastructure.get_mut(&id) {
                    infra.pending_investment += public_investment;
                }
            }

            if let Some(tech) = world.technology.get_mut(&id) {
                tech.current_allocation = rnd_allocation.max(0.0);
            }
        }
        Ok(())
    }
}
