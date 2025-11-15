use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use crate::world::WorldState;

#[derive(Clone, Debug)]
pub struct SnapshotConfig {
    pub interval: u64,
    pub output_dir: String,
}

impl SnapshotConfig {
    pub fn with_output_dir(mut self, dir: String) -> Self {
        self.output_dir = dir;
        self
    }
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            interval: 5,
            output_dir: "snapshots".to_string(),
        }
    }
}

pub struct SnapshotManager {
    config: SnapshotConfig,
}

impl SnapshotManager {
    pub fn new(config: SnapshotConfig) -> Self {
        Self { config }
    }

    pub fn maybe_snapshot(
        &self,
        tick: u64,
        scenario_name: &str,
        world: &WorldState,
    ) -> Result<Option<PathBuf>, SnapshotError> {
        if self.config.interval == 0 {
            return Ok(None);
        }

        if tick % self.config.interval != 0 {
            return Ok(None);
        }

        let dir = Path::new(&self.config.output_dir).join(scenario_name);
        fs::create_dir_all(&dir)?;
        let file_path = dir.join(format!("tick_{tick:06}.json"));
        let json = serialize_world(tick, world);
        fs::write(&file_path, json)?;
        Ok(Some(file_path))
    }
}

fn serialize_world(tick: u64, world: &WorldState) -> String {
    let mut json = String::new();
    json.push_str("{\n");
    json.push_str(&format!("  \"tick\": {tick},\n"));
    json.push_str("  \"tiles\": [\n");
    for (index, tile_state) in world.tiles.iter().enumerate() {
        json.push_str("    {\n");
        json.push_str(&format!("      \"id\": {},\n", tile_state.tile.id));
        json.push_str(&format!("      \"name\": \"{}\",\n", tile_state.tile.name));
        json.push_str("      \"environment\": {\n");
        json.push_str(&format!(
            "        \"temperature_c\": {:.4},\n",
            tile_state.environment.temperature_c
        ));
        json.push_str(&format!(
            "        \"precipitation_mm\": {:.4},\n",
            tile_state.environment.precipitation_mm
        ));
        json.push_str(&format!(
            "        \"fertility\": {:.4},\n",
            tile_state.environment.fertility
        ));
        json.push_str(&format!(
            "        \"seasonal_phase\": {:.4}\n",
            tile_state.environment.seasonal_phase
        ));
        json.push_str("      }\n");
        if index + 1 == world.tiles.len() {
            json.push_str("    }\n");
        } else {
            json.push_str("    },\n");
        }
    }
    json.push_str("  ],\n");
    json.push_str("  \"populations\": [\n");
    for (index, group) in world.populations.iter().enumerate() {
        json.push_str("    {\n");
        json.push_str(&format!("      \"tile_id\": {},\n", group.tile_id));
        json.push_str(&format!("      \"count\": {},\n", group.count));
        json.push_str(&format!(
            "      \"mean_age_years\": {:.2}\n",
            group.mean_age_years
        ));
        if index + 1 == world.populations.len() {
            json.push_str("    }\n");
        } else {
            json.push_str("    },\n");
        }
    }
    json.push_str("  ]\n");
    json.push_str("}\n");
    json
}

#[derive(Debug)]
pub enum SnapshotError {
    Io(std::io::Error),
}

impl From<std::io::Error> for SnapshotError {
    fn from(value: std::io::Error) -> Self {
        SnapshotError::Io(value)
    }
}

impl fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SnapshotError::Io(err) => write!(f, "snapshot io error: {err}"),
        }
    }
}

impl Error for SnapshotError {}
