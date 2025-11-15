//! PANARCHY - World Simulation Engine
//! Phase 0: Core Engine & Tiny Scenario

mod ecs;
mod rng;
mod scheduler;
mod spatial;
mod snapshot;
mod config;

use clap::{Parser, Subcommand};
use log::{info, debug};
use rand::Rng;
use std::path::PathBuf;
use std::time::Instant;

use ecs::World;
use rng::RngManager;
use scheduler::{Scheduler, BookkeepingSystem};
use spatial::{TileGrid, Location, Environment, LandCover, LandUse, ResourceStock};
use snapshot::SnapshotManager;
use config::Config;

#[derive(Parser)]
#[command(name = "panarchy")]
#[command(about = "World Simulation Engine", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a simulation scenario
    Run {
        /// Path to configuration file (YAML)
        #[arg(short, long, default_value = "scenarios/tiny_island.yaml")]
        config: PathBuf,
        
        /// Number of ticks to run
        #[arg(short, long, default_value = "100")]
        ticks: u64,
        
        /// Output directory for snapshots
        #[arg(short, long, default_value = "output")]
        output: PathBuf,
    },
    /// Generate a default scenario configuration
    GenerateConfig {
        /// Output file path
        #[arg(short, long, default_value = "scenarios/tiny_island.yaml")]
        output: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Run { config, ticks, output } => {
            run_simulation(&config, ticks, &output)?;
        }
        Commands::GenerateConfig { output } => {
            generate_config(&output)?;
        }
    }
    
    Ok(())
}

fn run_simulation(config_path: &PathBuf, num_ticks: u64, output_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    info!("Loading configuration from {:?}", config_path);
    
    let config = if config_path.exists() {
        Config::from_yaml(config_path)?
    } else {
        info!("Configuration file not found, using default tiny_island config");
        Config::tiny_island()
    };
    
    info!("Starting simulation: {}", config.name);
    info!("Random seed: {}", config.random_seed);
    info!("Spatial: {}x{} tiles", config.spatial.width_tiles, config.spatial.height_tiles);
    info!("Target population: {}", config.population.persons);
    
    // Initialize core components
    let mut world = World::new();
    let mut rng = RngManager::new(config.random_seed);
    let mut scheduler = Scheduler::new(1.0); // 1 day per tick
    
    // Add bookkeeping system
    scheduler.add_system(Box::new(BookkeepingSystem::new()));
    
    // Initialize snapshot manager
    let snapshot_dir = output_dir.join(&config.name);
    let mut snapshot_manager = SnapshotManager::new(&snapshot_dir, config.snapshot.every_ticks)?;
    
    // Initialize world
    info!("Initializing world...");
    let init_start = Instant::now();
    initialize_world(&mut world, &config, &mut rng);
    let init_time = init_start.elapsed();
    info!("World initialized in {:?}", init_time);
    info!("Total entities: {}", world.entity_count());
    
    // Run simulation
    info!("Running simulation for {} ticks...", num_ticks);
    let sim_start = Instant::now();
    
    for tick in 1..=num_ticks {
        let tick_stats = scheduler.tick(&mut world, &mut rng);
        
        // Take snapshots
        if snapshot_manager.should_snapshot(tick) {
            info!("Taking snapshot at tick {}...", tick);
            snapshot_manager.take_snapshot(&world, tick)?;
        }
        
        // Log progress
        if tick % 10 == 0 || tick == num_ticks {
            let avg_time = scheduler.average_tick_time().unwrap_or_default();
            info!("Tick {}/{} - Duration: {:?}, Avg: {:?}", 
                  tick, num_ticks, tick_stats.duration, avg_time);
            
            // Log system times
            for (name, duration) in &tick_stats.system_times {
                debug!("  System '{}': {:?}", name, duration);
            }
        }
    }
    
    let sim_time = sim_start.elapsed();
    let avg_tick_time = scheduler.average_tick_time().unwrap_or_default();
    
    info!("Simulation complete!");
    info!("Total time: {:?}", sim_time);
    info!("Average tick time: {:?}", avg_tick_time);
    info!("Ticks per second: {:.2}", num_ticks as f64 / sim_time.as_secs_f64());
    
    // Performance check for Phase 0 target (≤ 150ms/tick)
    let avg_ms = avg_tick_time.as_secs_f64() * 1000.0;
    if avg_ms <= 150.0 {
        info!("✓ Performance target met: {:.2}ms ≤ 150ms", avg_ms);
    } else {
        info!("✗ Performance target not met: {:.2}ms > 150ms", avg_ms);
    }
    
    Ok(())
}

fn initialize_world(world: &mut World, config: &Config, rng: &mut RngManager) {
    // Create tile grid
    let grid = TileGrid::new(config.spatial.width_tiles, config.spatial.height_tiles);
    let tile_count = grid.tile_count();
    
    info!("Creating {} tiles...", tile_count);
    
    // Create tile entities with components
    let mut tile_rng = rng.get_system_rng(0);
    
    for tile_id in 0..tile_count {
        let entity = world.create_entity();
        
        // Add location
        world.add_component(entity, Location { tile_id });
        
        // Add environment with some random variation
        world.add_component(entity, Environment {
            temp_idx: tile_rng.gen::<f32>() * 40.0 - 10.0, // -10 to 30 celsius
            precip_idx: tile_rng.gen::<f32>() * 2000.0,     // 0 to 2000mm
            soil_fertility: tile_rng.gen::<f32>(),          // 0 to 1
        });
        
        // Add land cover
        let land_use = if tile_rng.gen::<f32>() < 0.1 {
            LandUse::Urban
        } else if tile_rng.gen::<f32>() < 0.3 {
            LandUse::Forest
        } else if tile_rng.gen::<f32>() < 0.5 {
            LandUse::Cropland
        } else {
            LandUse::Grassland
        };
        
        world.add_component(entity, LandCover {
            land_use,
            fraction_urban: if land_use == LandUse::Urban { 0.8 } else { 0.1 },
            fraction_forest: if land_use == LandUse::Forest { 0.9 } else { 0.1 },
            fraction_cropland: if land_use == LandUse::Cropland { 0.8 } else { 0.0 },
        });
        
        // Add resource stock
        world.add_component(entity, ResourceStock {
            mineral_tonnage: tile_rng.gen::<f32>() * 1000000.0,
            ore_grade: tile_rng.gen::<f32>(),
            water_available: tile_rng.gen::<f32>() * 10000.0,
            biomass: tile_rng.gen::<f32>() * 1000.0,
        });
    }
    
    info!("Tiles created successfully");
}

fn generate_config(output_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    info!("Generating default tiny_island configuration...");
    
    // Create scenarios directory if it doesn't exist
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let config = Config::tiny_island();
    config.to_yaml(output_path)?;
    
    info!("Configuration saved to {:?}", output_path);
    Ok(())
}
