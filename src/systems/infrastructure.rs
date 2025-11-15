use anyhow::Result;

use crate::{
    engine::{System, SystemContext},
    rng::SystemRng,
    world::{EntityId, World},
};

const EPS: f64 = 1e-9;

pub struct InfrastructureSystem;

impl InfrastructureSystem {
    pub fn new() -> Self {
        Self
    }
}

impl Default for InfrastructureSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for InfrastructureSystem {
    fn name(&self) -> &str {
        "infrastructure"
    }

    fn run(
        &mut self,
        ctx: &SystemContext,
        world: &mut World,
        _rng: &mut SystemRng<'_>,
    ) -> Result<()> {
        let dt = ctx.dt_days;
        let mut ids: Vec<EntityId> = world.infrastructure.keys().cloned().collect();
        ids.sort();
        for id in ids {
            let economy_view = world.economies.get(&id).map(|econ| {
                (
                    econ.energy_shortage_ratio,
                    econ.transport_shortfall,
                    econ.energy_curtailed,
                    econ.energy_dispatched,
                )
            });
            let maintenance_cost = {
                let infra = match world.infrastructure.get_mut(&id) {
                    Some(infra) => infra,
                    None => continue,
                };
                let degrade = (infra.degradation_rate * dt).clamp(0.0, 0.5);
                infra.power_capacity = (infra.power_capacity * (1.0 - degrade)).max(0.0);
                infra.transport_capacity =
                    (infra.transport_capacity * (1.0 - degrade * 0.8)).max(0.0);

                let realized = (infra.pending_investment * 0.2).min(infra.pending_investment);
                if realized > 0.0 {
                    infra.power_capacity += realized * 0.6;
                    infra.transport_capacity += realized * 0.4;
                    infra.pending_investment -= realized;
                }

                if let Some((energy_shortage, transport_shortfall, curtailed, dispatched)) =
                    economy_view
                {
                    let energy_flow = (dispatched + curtailed).max(EPS);
                    let curtailed_share = (curtailed / energy_flow).clamp(0.0, 1.0);
                    let outage_penalty = (energy_shortage * 0.6)
                        + (transport_shortfall * 0.3)
                        + (curtailed_share * 0.1);
                    let reliability_score = (1.0 - outage_penalty).clamp(0.0, 1.0);
                    infra.reliability *= 1.0 - degrade * 0.4;
                    infra.reliability =
                        (infra.reliability * 0.7 + reliability_score * 0.3).clamp(0.0, 1.0);
                } else {
                    infra.reliability *= 1.0 - degrade * 0.4;
                }
                infra.maintenance_cost * dt
            };

            if maintenance_cost > 0.0 {
                if let Some(finance) = world.finances.get_mut(&id) {
                    if finance.bank_deposits >= maintenance_cost {
                        finance.bank_deposits -= maintenance_cost;
                    } else {
                        let remaining = maintenance_cost - finance.bank_deposits;
                        finance.bank_deposits = 0.0;
                        finance.loan_balance += remaining;
                    }
                }
            }
        }
        Ok(())
    }
}
