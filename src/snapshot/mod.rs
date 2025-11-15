//! Snapshot system for periodic state checkpoints

use crate::ecs::World;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Snapshot metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub tick: u64,
    pub timestamp: String,
    pub entity_count: usize,
}

/// Snapshot manager handles periodic checkpoints
pub struct SnapshotManager {
    output_dir: PathBuf,
    interval_ticks: u64,
    last_snapshot_tick: u64,
}

impl SnapshotManager {
    pub fn new<P: AsRef<Path>>(output_dir: P, interval_ticks: u64) -> std::io::Result<Self> {
        let output_dir = output_dir.as_ref().to_path_buf();
        fs::create_dir_all(&output_dir)?;

        Ok(Self {
            output_dir,
            interval_ticks,
            last_snapshot_tick: 0,
        })
    }

    /// Check if a snapshot should be taken this tick
    pub fn should_snapshot(&self, current_tick: u64) -> bool {
        if self.interval_ticks == 0 {
            return false;
        }
        current_tick > 0 && current_tick - self.last_snapshot_tick >= self.interval_ticks
    }

    /// Take a snapshot of the world state
    pub fn take_snapshot(&mut self, world: &World, tick: u64) -> std::io::Result<PathBuf> {
        let metadata = SnapshotMetadata {
            tick,
            timestamp: chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string(),
            entity_count: world.entity_count(),
        };

        let snapshot_dir = self.output_dir.join(format!("snapshot_{:08}", tick));
        fs::create_dir_all(&snapshot_dir)?;

        // Save metadata
        let metadata_path = snapshot_dir.join("metadata.json");
        let metadata_json = serde_json::to_string_pretty(&metadata)?;
        let mut file = File::create(&metadata_path)?;
        file.write_all(metadata_json.as_bytes())?;

        // In Phase 0, we just save basic metadata
        // In later phases, we would serialize component data to Parquet

        self.last_snapshot_tick = tick;
        Ok(snapshot_dir)
    }

    /// Load a snapshot (placeholder for Phase 0)
    pub fn load_snapshot<P: AsRef<Path>>(&self, snapshot_dir: P) -> std::io::Result<SnapshotMetadata> {
        let metadata_path = snapshot_dir.as_ref().join("metadata.json");
        let mut file = File::open(metadata_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        let metadata: SnapshotMetadata = serde_json::from_str(&contents)?;
        Ok(metadata)
    }

    /// List available snapshots
    pub fn list_snapshots(&self) -> std::io::Result<Vec<PathBuf>> {
        let mut snapshots = Vec::new();
        
        if !self.output_dir.exists() {
            return Ok(snapshots);
        }

        for entry in fs::read_dir(&self.output_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() && path.file_name().unwrap().to_str().unwrap().starts_with("snapshot_") {
                snapshots.push(path);
            }
        }

        snapshots.sort();
        Ok(snapshots)
    }
}

// Add chrono for timestamps (we'll add this to Cargo.toml later)
mod chrono {
    pub struct Local;
    impl Local {
        pub fn now() -> DateTime {
            DateTime
        }
    }
    
    pub struct DateTime;
    impl DateTime {
        pub fn format(&self, _fmt: &str) -> String {
            use std::time::SystemTime;
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap();
            format!("{}", now.as_secs())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_snapshot_should_snapshot() {
        let temp_dir = env::temp_dir().join("panarchy_test_snapshots");
        let manager = SnapshotManager::new(&temp_dir, 30).unwrap();

        assert!(!manager.should_snapshot(0));
        assert!(!manager.should_snapshot(29));
        assert!(manager.should_snapshot(30));
        assert!(manager.should_snapshot(31));
    }

    #[test]
    fn test_snapshot_creation() {
        let temp_dir = env::temp_dir().join("panarchy_test_snapshots_2");
        let mut manager = SnapshotManager::new(&temp_dir, 30).unwrap();
        let mut world = World::new();

        // Create some entities
        world.create_entity();
        world.create_entity();

        let snapshot_dir = manager.take_snapshot(&world, 30).unwrap();
        assert!(snapshot_dir.exists());

        let metadata = manager.load_snapshot(&snapshot_dir).unwrap();
        assert_eq!(metadata.tick, 30);
        assert_eq!(metadata.entity_count, 2);

        // Cleanup
        fs::remove_dir_all(&temp_dir).ok();
    }
}
