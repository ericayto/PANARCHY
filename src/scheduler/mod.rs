//! Scheduler - manages tick loop and system execution

use crate::ecs::World;
use crate::rng::RngManager;
use std::time::{Duration, Instant};

/// System trait - each subsystem implements this
pub trait System: Send + Sync {
    fn name(&self) -> &str;
    fn system_id(&self) -> u32;
    fn update(&mut self, world: &mut World, rng: &mut RngManager, dt_days: f64);
}

/// Statistics for a single tick
#[derive(Debug, Clone)]
pub struct TickStats {
    pub tick: u64,
    pub duration: Duration,
    pub system_times: Vec<(String, Duration)>,
}

/// Scheduler manages the simulation loop
pub struct Scheduler {
    systems: Vec<Box<dyn System>>,
    tick_count: u64,
    dt_days: f64,
    stats_history: Vec<TickStats>,
    max_stats_history: usize,
}

impl Scheduler {
    pub fn new(dt_days: f64) -> Self {
        Self {
            systems: Vec::new(),
            tick_count: 0,
            dt_days,
            stats_history: Vec::new(),
            max_stats_history: 100,
        }
    }

    /// Add a system to the scheduler
    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
    }

    /// Get current tick count
    pub fn tick_count(&self) -> u64 {
        self.tick_count
    }

    /// Get dt in days
    pub fn dt_days(&self) -> f64 {
        self.dt_days
    }

    /// Execute one tick
    pub fn tick(&mut self, world: &mut World, rng: &mut RngManager) -> TickStats {
        let tick_start = Instant::now();
        let mut system_times = Vec::new();

        // Advance RNG tick
        rng.advance_tick();

        // Execute all systems in order
        for system in &mut self.systems {
            let system_start = Instant::now();
            system.update(world, rng, self.dt_days);
            let system_duration = system_start.elapsed();
            system_times.push((system.name().to_string(), system_duration));
        }

        self.tick_count += 1;
        let tick_duration = tick_start.elapsed();

        let stats = TickStats {
            tick: self.tick_count,
            duration: tick_duration,
            system_times,
        };

        // Store stats
        self.stats_history.push(stats.clone());
        if self.stats_history.len() > self.max_stats_history {
            self.stats_history.remove(0);
        }

        stats
    }

    /// Get recent tick statistics
    pub fn recent_stats(&self) -> &[TickStats] {
        &self.stats_history
    }

    /// Get average tick time from recent history
    pub fn average_tick_time(&self) -> Option<Duration> {
        if self.stats_history.is_empty() {
            return None;
        }

        let total: Duration = self.stats_history.iter().map(|s| s.duration).sum();
        Some(total / self.stats_history.len() as u32)
    }

    /// Run simulation for a number of ticks
    pub fn run(&mut self, world: &mut World, rng: &mut RngManager, num_ticks: u64) {
        for _ in 0..num_ticks {
            self.tick(world, rng);
        }
    }
}

/// A simple bookkeeping system for Phase 0
pub struct BookkeepingSystem {
    system_id: u32,
}

impl BookkeepingSystem {
    pub fn new() -> Self {
        Self { system_id: 999 }
    }
}

impl Default for BookkeepingSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for BookkeepingSystem {
    fn name(&self) -> &str {
        "bookkeeping"
    }

    fn system_id(&self) -> u32 {
        self.system_id
    }

    fn update(&mut self, _world: &mut World, _rng: &mut RngManager, _dt_days: f64) {
        // For Phase 0, this is a placeholder
        // In later phases, this would check invariants and update KPIs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestSystem {
        id: u32,
        call_count: u32,
    }

    impl System for TestSystem {
        fn name(&self) -> &str {
            "test_system"
        }

        fn system_id(&self) -> u32 {
            self.id
        }

        fn update(&mut self, _world: &mut World, _rng: &mut RngManager, _dt_days: f64) {
            self.call_count += 1;
        }
    }

    #[test]
    fn test_scheduler_ticks() {
        let mut scheduler = Scheduler::new(1.0);
        let mut world = World::new();
        let mut rng = RngManager::new(42);

        let system = Box::new(TestSystem { id: 1, call_count: 0 });
        scheduler.add_system(system);

        assert_eq!(scheduler.tick_count(), 0);

        scheduler.tick(&mut world, &mut rng);
        assert_eq!(scheduler.tick_count(), 1);
        assert_eq!(rng.current_tick(), 1);

        scheduler.tick(&mut world, &mut rng);
        assert_eq!(scheduler.tick_count(), 2);
        assert_eq!(rng.current_tick(), 2);
    }

    #[test]
    fn test_scheduler_stats() {
        let mut scheduler = Scheduler::new(1.0);
        let mut world = World::new();
        let mut rng = RngManager::new(42);

        let system = Box::new(TestSystem { id: 1, call_count: 0 });
        scheduler.add_system(system);

        scheduler.tick(&mut world, &mut rng);

        let stats = scheduler.recent_stats();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].tick, 1);
    }
}
