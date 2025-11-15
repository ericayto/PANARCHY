use std::collections::HashSet;

#[derive(Debug, Clone, Copy)]
pub struct TechDefinition {
    pub id: &'static str,
    pub display: &'static str,
    pub difficulty: f64,
    pub food_multiplier: f64,
    pub energy_multiplier: f64,
    pub prerequisites: &'static [&'static str],
}

const TECH_TREE: &[TechDefinition] = &[
    TechDefinition {
        id: "adaptive_farming",
        display: "Adaptive Farming",
        difficulty: 12_000.0,
        food_multiplier: 1.08,
        energy_multiplier: 1.0,
        prerequisites: &[],
    },
    TechDefinition {
        id: "grid_storage",
        display: "Grid Storage",
        difficulty: 16_000.0,
        food_multiplier: 1.0,
        energy_multiplier: 1.12,
        prerequisites: &["adaptive_farming"],
    },
    TechDefinition {
        id: "automation_lines",
        display: "Automation Lines",
        difficulty: 22_500.0,
        food_multiplier: 1.06,
        energy_multiplier: 1.05,
        prerequisites: &["adaptive_farming"],
    },
    TechDefinition {
        id: "circular_economy",
        display: "Circular Economy",
        difficulty: 30_000.0,
        food_multiplier: 1.04,
        energy_multiplier: 1.08,
        prerequisites: &["grid_storage", "automation_lines"],
    },
];

pub fn definition(id: &str) -> Option<&'static TechDefinition> {
    TECH_TREE.iter().find(|def| def.id == id)
}

pub fn next_available(unlocked: &[String]) -> Option<&'static TechDefinition> {
    let unlocked: HashSet<&str> = unlocked.iter().map(|s| s.as_str()).collect();
    TECH_TREE.iter().find(|def| {
        if unlocked.contains(def.id) {
            return false;
        }
        def.prerequisites.iter().all(|dep| unlocked.contains(*dep))
    })
}

pub fn aggregate_productivity_multipliers(unlocked: &[String]) -> (f64, f64) {
    let mut food = 1.0;
    let mut energy = 1.0;
    for tech in unlocked {
        if let Some(def) = definition(tech) {
            food *= def.food_multiplier;
            energy *= def.energy_multiplier;
        }
    }
    (food, energy)
}
