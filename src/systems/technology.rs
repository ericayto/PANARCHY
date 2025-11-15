use anyhow::Result;

use crate::{
    engine::{System, SystemContext},
    rng::SystemRng,
    technology,
    world::{EntityId, ResearchProject, World},
};

pub struct TechnologySystem;

impl TechnologySystem {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TechnologySystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for TechnologySystem {
    fn name(&self) -> &str {
        "technology"
    }

    fn run(
        &mut self,
        ctx: &SystemContext,
        world: &mut World,
        _rng: &mut SystemRng<'_>,
    ) -> Result<()> {
        let dt = ctx.dt_days;
        let mut ids: Vec<EntityId> = world.technology.keys().cloned().collect();
        ids.sort();
        for id in ids {
            let mut updated_productivity: Option<(f64, f64)> = None;
            if let Some(tech) = world.technology.get_mut(&id) {
                let (food_mult, energy_mult) =
                    technology::aggregate_productivity_multipliers(&tech.unlocked);
                updated_productivity = Some((
                    tech.base_food_productivity * food_mult,
                    tech.base_energy_productivity * energy_mult,
                ));
                let allocation = tech.current_allocation.max(0.0);
                if allocation <= 0.0 {
                    tech.innovation_score *= 0.9;
                } else {
                    if tech.active_project.is_none() {
                        if let Some(def) = technology::next_available(&tech.unlocked) {
                            tech.active_project = Some(ResearchProject {
                                tech_id: def.id.to_string(),
                                progress: 0.0,
                                difficulty: def.difficulty,
                            });
                        }
                    }
                    if let Some(project) = tech.active_project.as_mut() {
                        let progress_gain = allocation * tech.research_efficiency * dt;
                        project.progress += progress_gain;
                        tech.innovation_score = (tech.innovation_score * 0.7 + progress_gain * 0.3)
                            .clamp(0.0, f64::MAX);
                        if project.progress >= project.difficulty {
                            if !tech.unlocked.iter().any(|t| t == &project.tech_id) {
                                tech.unlocked.push(project.tech_id.clone());
                            }
                            tech.active_project = None;
                        }
                    } else {
                        tech.innovation_score *= 0.95;
                    }
                }
            }
            if let Some((food, energy)) = updated_productivity {
                if let Some(econ) = world.economies.get_mut(&id) {
                    econ.food_productivity_per_worker = food;
                    econ.energy_productivity_per_worker = energy;
                }
            }
        }
        Ok(())
    }
}
