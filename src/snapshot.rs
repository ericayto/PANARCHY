use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;

use crate::world::World;

pub struct SnapshotWriter {
    output_dir: PathBuf,
    interval_ticks: u64,
}

impl SnapshotWriter {
    pub fn new(root: impl AsRef<Path>, interval_ticks: u64) -> Self {
        Self {
            output_dir: root.as_ref().to_path_buf(),
            interval_ticks,
        }
    }

    pub fn maybe_write(&self, world: &World, scenario: &str) -> Result<()> {
        if self.interval_ticks == 0 || world.tick() == 0 {
            return Ok(());
        }

        if world.tick() % self.interval_ticks != 0 {
            return Ok(());
        }

        let snapshot_dir = self.output_dir.join(scenario);
        fs::create_dir_all(&snapshot_dir)?;
        let filename = snapshot_dir.join(format!("tick_{:06}.json", world.tick()));
        let payload = serde_json::to_vec_pretty(&world.snapshot(scenario))?;
        fs::write(filename, payload)?;
        Ok(())
    }
}
