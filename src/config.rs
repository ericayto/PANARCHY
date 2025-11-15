use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;

use crate::snapshot::SnapshotConfig;
use crate::world::WorldState;

#[derive(Clone, Debug)]
pub struct Scenario {
    pub name: String,
    pub random_seed: u64,
    pub ticks: u64,
    pub tick_days: f64,
    pub snapshot: SnapshotConfig,
    pub world: WorldConfig,
}

#[derive(Clone, Debug, Default)]
pub struct WorldConfig {
    pub tiles: Vec<TileConfig>,
    pub populations: Vec<PopulationConfig>,
}

#[derive(Clone, Debug)]
pub struct TileConfig {
    pub id: u32,
    pub name: String,
    pub temperature_c: f32,
    pub precipitation_mm: f32,
    pub fertility: f32,
}

#[derive(Clone, Debug)]
pub struct PopulationConfig {
    pub tile_id: u32,
    pub count: u32,
    pub mean_age_years: f32,
}

#[derive(Debug)]
pub enum ScenarioError {
    Io(std::io::Error),
    Parse(String),
    Validation(String),
}

impl From<std::io::Error> for ScenarioError {
    fn from(value: std::io::Error) -> Self {
        ScenarioError::Io(value)
    }
}

impl fmt::Display for ScenarioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScenarioError::Io(err) => write!(f, "{err}"),
            ScenarioError::Parse(msg) => write!(f, "scenario parse error: {msg}"),
            ScenarioError::Validation(msg) => write!(f, "scenario validation error: {msg}"),
        }
    }
}

impl Error for ScenarioError {}

impl Scenario {
    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, ScenarioError> {
        let text = fs::read_to_string(path)?;
        Self::from_str(&text)
    }

    pub fn from_str(text: &str) -> Result<Self, ScenarioError> {
        Parser::new(text).parse()
    }

    pub fn validate(&self) -> Result<(), ScenarioError> {
        if self.world.tiles.is_empty() {
            return Err(ScenarioError::Validation(
                "scenario must define at least one tile".to_string(),
            ));
        }

        let mut known_tiles = Vec::new();
        for tile in &self.world.tiles {
            if known_tiles.contains(&tile.id) {
                return Err(ScenarioError::Validation(format!(
                    "tile id {} defined more than once",
                    tile.id
                )));
            }
            known_tiles.push(tile.id);
        }

        if self.world.populations.is_empty() {
            return Err(ScenarioError::Validation(
                "scenario must define at least one population group".into(),
            ));
        }

        for population in &self.world.populations {
            if !known_tiles.contains(&population.tile_id) {
                return Err(ScenarioError::Validation(format!(
                    "population references unknown tile id {}",
                    population.tile_id
                )));
            }
        }

        if self.total_population() == 0 {
            return Err(ScenarioError::Validation(
                "total population must be greater than zero".into(),
            ));
        }

        Ok(())
    }

    pub fn total_population(&self) -> u64 {
        self.world
            .populations
            .iter()
            .map(|group| group.count as u64)
            .sum()
    }

    pub fn build_world(&self) -> WorldState {
        let mut world = WorldState::default();
        for tile in &self.world.tiles {
            world.add_tile(
                crate::components::Tile {
                    id: tile.id,
                    name: tile.name.clone(),
                },
                crate::components::Environment::new(
                    tile.temperature_c,
                    tile.precipitation_mm,
                    tile.fertility,
                ),
            );
        }
        for population in &self.world.populations {
            world.add_population(crate::components::PopulationGroup::new(
                population.tile_id,
                population.count,
                population.mean_age_years,
            ));
        }
        world
    }
}

struct Parser<'a> {
    lines: Vec<&'a str>,
    index: usize,
}

impl<'a> Parser<'a> {
    fn new(text: &'a str) -> Self {
        let lines = text.lines().collect();
        Self { lines, index: 0 }
    }

    fn parse(&mut self) -> Result<Scenario, ScenarioError> {
        let mut name = String::new();
        let mut random_seed = 0_u64;
        let mut ticks = 0_u64;
        let mut tick_days = 1.0_f64;
        let mut snapshot = SnapshotConfig::default();
        let mut world = WorldConfig::default();
        let mut context = Context::Root;

        while let Some(line) = self.next_line() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            match trimmed {
                "snapshot:" => {
                    context = Context::Snapshot;
                    continue;
                }
                "world:" => {
                    context = Context::World;
                    continue;
                }
                "tiles:" => {
                    context = Context::Tiles;
                    continue;
                }
                "populations:" => {
                    context = Context::Populations;
                    continue;
                }
                _ => {}
            }

            match context {
                Context::Root => {
                    if let Some(value) = trimmed.strip_prefix("name:") {
                        name = clean_string(value);
                    } else if let Some(value) = trimmed.strip_prefix("random_seed:") {
                        random_seed = parse_u64(value)?;
                    } else if let Some(value) = trimmed.strip_prefix("ticks:") {
                        ticks = parse_u64(value)?;
                    } else if let Some(value) = trimmed.strip_prefix("tick_days:") {
                        tick_days = parse_f64(value)?;
                    }
                }
                Context::Snapshot => {
                    if let Some(value) = trimmed.strip_prefix("interval:") {
                        snapshot.interval = parse_u64(value)?;
                    } else if let Some(value) = trimmed.strip_prefix("output_dir:") {
                        snapshot.output_dir = clean_string(value);
                    }
                }
                Context::World => {}
                Context::Tiles => {
                    if let Some(value) = trimmed.strip_prefix("- id:") {
                        let id = parse_u32(value)?;
                        world.tiles.push(TileConfig {
                            id,
                            name: String::new(),
                            temperature_c: 0.0,
                            precipitation_mm: 0.0,
                            fertility: 0.0,
                        });
                    } else if let Some(tile) = world.tiles.last_mut() {
                        if let Some(value) = trimmed.strip_prefix("name:") {
                            tile.name = clean_string(value);
                        } else if let Some(value) = trimmed.strip_prefix("temperature_c:") {
                            tile.temperature_c = parse_f32(value)?;
                        } else if let Some(value) = trimmed.strip_prefix("precipitation_mm:") {
                            tile.precipitation_mm = parse_f32(value)?;
                        } else if let Some(value) = trimmed.strip_prefix("fertility:") {
                            tile.fertility = parse_f32(value)?;
                        }
                    }
                }
                Context::Populations => {
                    if let Some(value) = trimmed.strip_prefix("- tile_id:") {
                        let tile_id = parse_u32(value)?;
                        world.populations.push(PopulationConfig {
                            tile_id,
                            count: 0,
                            mean_age_years: 0.0,
                        });
                    } else if let Some(pop) = world.populations.last_mut() {
                        if let Some(value) = trimmed.strip_prefix("count:") {
                            pop.count = parse_u32(value)?;
                        } else if let Some(value) = trimmed.strip_prefix("mean_age_years:") {
                            pop.mean_age_years = parse_f32(value)?;
                        }
                    }
                }
            }
        }

        if name.is_empty() {
            return Err(ScenarioError::Parse(
                "scenario must define a name".to_string(),
            ));
        }

        if ticks == 0 {
            ticks = 30;
        }

        let scenario = Scenario {
            name,
            random_seed,
            ticks,
            tick_days,
            snapshot,
            world,
        };
        scenario.validate()?;
        Ok(scenario)
    }

    fn next_line(&mut self) -> Option<&'a str> {
        if self.index >= self.lines.len() {
            None
        } else {
            let line = self.lines[self.index];
            self.index += 1;
            Some(line)
        }
    }
}

enum Context {
    Root,
    Snapshot,
    World,
    Tiles,
    Populations,
}

fn parse_u64(value: &str) -> Result<u64, ScenarioError> {
    value
        .trim()
        .parse::<u64>()
        .map_err(|_| ScenarioError::Parse(format!("unable to parse integer from '{value}'")))
}

fn parse_u32(value: &str) -> Result<u32, ScenarioError> {
    value
        .trim()
        .parse::<u32>()
        .map_err(|_| ScenarioError::Parse(format!("unable to parse integer from '{value}'")))
}

fn parse_f64(value: &str) -> Result<f64, ScenarioError> {
    value
        .trim()
        .parse::<f64>()
        .map_err(|_| ScenarioError::Parse(format!("unable to parse float from '{value}'")))
}

fn parse_f32(value: &str) -> Result<f32, ScenarioError> {
    value
        .trim()
        .parse::<f32>()
        .map_err(|_| ScenarioError::Parse(format!("unable to parse float from '{value}'")))
}

fn clean_string(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}
