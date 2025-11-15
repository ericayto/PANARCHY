use anyhow::Result;

use crate::{
    engine::{System, SystemContext},
    rng::SystemRng,
    world::{EntityId, World},
};

const EPS: f64 = 1e-9;

pub struct FinanceSystem;

impl FinanceSystem {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FinanceSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for FinanceSystem {
    fn name(&self) -> &str {
        "finance"
    }

    fn run(
        &mut self,
        ctx: &SystemContext,
        world: &mut World,
        _rng: &mut SystemRng<'_>,
    ) -> Result<()> {
        let dt_years = ctx.dt_days / 365.0;
        let mut ids: Vec<EntityId> = world.finances.keys().cloned().collect();
        ids.sort();
        for id in ids {
            let economy_snapshot = match world.economies.get(&id) {
                Some(econ) => (
                    econ.sales_revenue,
                    econ.wage_bill,
                    econ.food_shortage_ratio,
                    econ.energy_shortage_ratio,
                    econ.transport_shortfall,
                ),
                None => continue,
            };
            let (revenue, wage_bill, food_shortage, energy_shortage, transport_shortfall) =
                economy_snapshot;
            let mut infra_investment = 0.0;
            {
                let finance = match world.finances.get_mut(&id) {
                    Some(finance) => finance,
                    None => continue,
                };
                let mut net_cash = revenue - wage_bill;
                if !net_cash.is_finite() {
                    net_cash = 0.0;
                }
                if net_cash >= 0.0 {
                    infra_investment = net_cash * finance.infrastructure_spend_fraction;
                    finance.bank_deposits += net_cash - infra_investment;
                } else {
                    let mut need = -net_cash;
                    if finance.bank_deposits >= need {
                        finance.bank_deposits -= need;
                    } else {
                        need -= finance.bank_deposits;
                        finance.bank_deposits = 0.0;
                        finance.loan_balance += need;
                    }
                }

                let loan_rate = (finance.policy_rate + finance.loan_rate_spread).max(0.0);
                if finance.loan_balance > 0.0 {
                    finance.loan_balance *= 1.0 + loan_rate * dt_years;
                }
                if finance.bank_deposits > 0.0 {
                    finance.bank_deposits *= 1.0 + finance.deposit_rate.max(0.0) * dt_years;
                }

                let stress_signal =
                    (food_shortage + energy_shortage) * 0.5 + transport_shortfall * 0.5;
                let stress = stress_signal.clamp(0.0, 2.0);
                finance.credit_stress = finance.credit_stress * 0.85 + stress * 0.15;

                let default_rate = finance.default_rate * (1.0 + finance.credit_stress);
                if finance.loan_balance > 0.0 {
                    let defaults = finance.loan_balance * default_rate * dt_years;
                    finance.loan_balance = (finance.loan_balance - defaults).max(0.0);
                    finance.cumulative_defaults += defaults;
                }

                let loan_to_deposit = if finance.bank_deposits > EPS {
                    finance.loan_balance / finance.bank_deposits
                } else if finance.loan_balance > 0.0 {
                    f64::INFINITY
                } else {
                    0.0
                };
                if loan_to_deposit.is_finite() && loan_to_deposit > finance.target_loan_to_deposit {
                    let pressure =
                        (loan_to_deposit / finance.target_loan_to_deposit - 1.0).min(1.0);
                    finance.loan_rate_spread *= 1.0 + 0.05 * pressure;
                } else {
                    finance.loan_rate_spread *= 0.995;
                }
                finance.loan_rate_spread = finance.loan_rate_spread.clamp(0.0, 0.5);
            }

            if infra_investment > 0.0 {
                if let Some(infra) = world.infrastructure.get_mut(&id) {
                    infra.pending_investment += infra_investment;
                }
            }
        }
        Ok(())
    }
}
