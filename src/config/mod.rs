//! Configuration module for scenario setup

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Main configuration for a simulation scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub random_seed: u64,
    pub spatial: SpatialConfig,
    pub population: PopulationConfig,
    pub snapshot: SnapshotConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialConfig {
    pub level: u32,
    pub width_tiles: u32,
    pub height_tiles: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationConfig {
    pub persons: u64,
    #[serde(default)]
    pub use_representative_households: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotConfig {
    pub every_ticks: u64,
    #[serde(default = "default_compression")]
    pub compression: String,
}

fn default_compression() -> String {
    "none".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_kpi_interval")]
    pub kpi_interval_days: u64,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_kpi_interval() -> u64 {
    30
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            kpi_interval_days: default_kpi_interval(),
        }
    }
}

impl Config {
    /// Load configuration from YAML file
    pub fn from_yaml<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        let config: Config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }

    /// Save configuration to YAML file
    pub fn to_yaml<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let yaml = serde_yaml::to_string(self)?;
        std::fs::write(path, yaml)?;
        Ok(())
    }

    /// Create the default tiny_island configuration
    pub fn tiny_island() -> Self {
        Self {
            name: "tiny_island".to_string(),
            random_seed: 7,
            spatial: SpatialConfig {
                level: 7,
                width_tiles: 128,
                height_tiles: 64,
            },
            population: PopulationConfig {
                persons: 50_000,
                use_representative_households: true,
            },
            snapshot: SnapshotConfig {
                every_ticks: 30,
                compression: "none".to_string(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                kpi_interval_days: 30,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_tiny_island_config() {
        let config = Config::tiny_island();
        
        assert_eq!(config.name, "tiny_island");
        assert_eq!(config.random_seed, 7);
        assert_eq!(config.spatial.width_tiles, 128);
        assert_eq!(config.spatial.height_tiles, 64);
        assert_eq!(config.population.persons, 50_000);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::tiny_island();
        
        let temp_file = env::temp_dir().join("test_config.yaml");
        config.to_yaml(&temp_file).unwrap();
        
        let loaded_config = Config::from_yaml(&temp_file).unwrap();
        assert_eq!(config.name, loaded_config.name);
        assert_eq!(config.random_seed, loaded_config.random_seed);
        
        std::fs::remove_file(&temp_file).ok();
    }
}
