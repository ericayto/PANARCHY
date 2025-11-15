//! Deterministic random number generation
//! 
//! Uses counter-based RNG with seeds derived from (system_id, entity_id, tick)

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;

/// System identifier for RNG streams
pub type SystemId = u32;

/// Global RNG state
pub struct RngManager {
    master_seed: u64,
    current_tick: u64,
    system_rngs: HashMap<SystemId, ChaCha8Rng>,
}

impl RngManager {
    pub fn new(seed: u64) -> Self {
        Self {
            master_seed: seed,
            current_tick: 0,
            system_rngs: HashMap::new(),
        }
    }

    /// Advance to the next tick
    pub fn advance_tick(&mut self) {
        self.current_tick += 1;
        self.system_rngs.clear();
    }

    /// Get current tick
    pub fn current_tick(&self) -> u64 {
        self.current_tick
    }

    /// Get or create RNG for a specific system
    pub fn get_system_rng(&mut self, system_id: SystemId) -> &mut ChaCha8Rng {
        let current_tick = self.current_tick;
        let seed = self.derive_seed(system_id, 0, current_tick);
        self.system_rngs.entry(system_id).or_insert_with(|| {
            ChaCha8Rng::seed_from_u64(seed)
        })
    }

    /// Create a deterministic RNG for a specific (system, entity, tick)
    pub fn create_entity_rng(&self, system_id: SystemId, entity_id: u64) -> ChaCha8Rng {
        let seed = self.derive_seed(system_id, entity_id, self.current_tick);
        ChaCha8Rng::seed_from_u64(seed)
    }

    /// Derive a seed from system_id, entity_id, and tick
    fn derive_seed(&self, system_id: SystemId, entity_id: u64, tick: u64) -> u64 {
        // Mix the master seed with system_id, entity_id, and tick
        let mut seed = self.master_seed;
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        seed ^= (system_id as u64).wrapping_mul(1103515245);
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        seed ^= entity_id.wrapping_mul(48271);
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        seed ^= tick.wrapping_mul(69069);
        seed
    }
}

impl Default for RngManager {
    fn default() -> Self {
        Self::new(42)
    }
}

/// Helper functions for common random operations
pub trait RngExt {
    fn random_f32(&mut self, min: f32, max: f32) -> f32;
    fn random_bool(&mut self, probability: f32) -> bool;
}

impl<R: Rng> RngExt for R {
    fn random_f32(&mut self, min: f32, max: f32) -> f32 {
        self.gen::<f32>() * (max - min) + min
    }

    fn random_bool(&mut self, probability: f32) -> bool {
        self.gen::<f32>() < probability
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_rng() {
        let mut rng1 = RngManager::new(42);
        let mut rng2 = RngManager::new(42);

        let val1: f32 = rng1.get_system_rng(1).gen();
        let val2: f32 = rng2.get_system_rng(1).gen();

        assert_eq!(val1, val2, "Same seed should produce same values");
    }

    #[test]
    fn test_tick_advance() {
        let mut rng = RngManager::new(42);
        
        assert_eq!(rng.current_tick(), 0);
        
        let val1: f32 = rng.get_system_rng(1).gen();
        
        rng.advance_tick();
        assert_eq!(rng.current_tick(), 1);
        
        let val2: f32 = rng.get_system_rng(1).gen();
        
        // Different ticks should produce different values
        assert_ne!(val1, val2);
    }

    #[test]
    fn test_different_systems_different_values() {
        let mut rng = RngManager::new(42);
        
        let val1: f32 = rng.get_system_rng(1).gen();
        let val2: f32 = rng.get_system_rng(2).gen();
        
        // Different systems should produce different values
        assert_ne!(val1, val2);
    }

    #[test]
    fn test_entity_rng() {
        let rng = RngManager::new(42);
        
        let mut rng1 = rng.create_entity_rng(1, 100);
        let mut rng2 = rng.create_entity_rng(1, 100);
        let mut rng3 = rng.create_entity_rng(1, 101);
        
        let val1: f32 = rng1.gen();
        let val2: f32 = rng2.gen();
        let val3: f32 = rng3.gen();
        
        assert_eq!(val1, val2, "Same entity should produce same values");
        assert_ne!(val1, val3, "Different entities should produce different values");
    }
}
