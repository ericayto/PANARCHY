use std::collections::HashMap;

use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;

pub struct RngManager {
    master: ChaCha8Rng,
    streams: HashMap<String, ChaCha8Rng>,
}

impl RngManager {
    pub fn new(seed: u64) -> Self {
        Self {
            master: ChaCha8Rng::seed_from_u64(seed),
            streams: HashMap::new(),
        }
    }

    pub fn stream(&mut self, name: &str) -> SystemRng<'_> {
        use rand::RngCore;
        let entry = self.streams.entry(name.to_string()).or_insert_with(|| {
            let mut seed_bytes = [0u8; 32];
            self.master.fill_bytes(&mut seed_bytes);
            let mut seed_u64 = [0u8; 8];
            seed_u64.copy_from_slice(&seed_bytes[..8]);
            let derived = u64::from_le_bytes(seed_u64);
            ChaCha8Rng::seed_from_u64(derived)
        });
        SystemRng { inner: entry }
    }
}

pub struct SystemRng<'a> {
    inner: &'a mut ChaCha8Rng,
}

impl<'a> RngCore for SystemRng<'a> {
    fn next_u32(&mut self) -> u32 {
        self.inner.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.inner.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.inner.fill_bytes(dest);
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.inner.try_fill_bytes(dest)
    }
}
