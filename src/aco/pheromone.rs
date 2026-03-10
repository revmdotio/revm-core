use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use super::config::AcoConfig;

/// Thread-safe pheromone matrix using lock-free atomic operations.
///
/// v0.3.0: Replaced `RwLock<Vec<f64>>` with `AtomicU64` per edge.
/// Each pheromone value is stored as `f64::to_bits() -> u64` and updated
/// via Compare-And-Swap (CAS) loops. This eliminates mutex contention
/// under high-throughput conditions (>5k tx/s).
///
/// Based on Herlihy & Shavit (2012) non-blocking concurrent data structures,
/// adapted for continuous-valued pheromone fields.
#[derive(Debug)]
pub struct PheromoneMatrix {
    size: usize,
    data: Arc<Vec<AtomicU64>>,
    config: AcoConfig,
}

impl Clone for PheromoneMatrix {
    fn clone(&self) -> Self {
        let new_data: Vec<AtomicU64> = self
            .data
            .iter()
            .map(|a| AtomicU64::new(a.load(Ordering::Relaxed)))
            .collect();
        Self {
            size: self.size,
            data: Arc::new(new_data),
            config: self.config.clone(),
        }
    }
}

/// Snapshot of the pheromone matrix for serialization and diagnostics.
#[derive(Debug, Serialize, Deserialize)]
pub struct PheromoneSnapshot {
    pub size: usize,
    pub edges: Vec<PheromoneEdge>,
    pub total_pheromone: f64,
    pub avg_pheromone: f64,
    pub timestamp_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PheromoneEdge {
    pub from: usize,
    pub to: usize,
    pub intensity: f64,
}

impl PheromoneMatrix {
    pub fn new(size: usize, config: &AcoConfig) -> Self {
        let initial_bits = config.initial_pheromone.to_bits();
        let data: Vec<AtomicU64> = (0..size * size)
            .map(|_| AtomicU64::new(initial_bits))
            .collect();
        Self {
            size,
            data: Arc::new(data),
            config: config.clone(),
        }
    }

    /// Get pheromone intensity on edge (from -> to).
    /// Lock-free atomic load.
    #[inline]
    pub fn get(&self, from: usize, to: usize) -> f64 {
        let bits = self.data[from * self.size + to].load(Ordering::Relaxed);
        f64::from_bits(bits)
    }

    /// Set pheromone intensity on edge (from -> to) directly.
    /// Lock-free atomic store with MMAS clamping.
    #[inline]
    pub fn set(&self, from: usize, to: usize, value: f64) {
        let clamped = value.clamp(self.config.pheromone_min, self.config.pheromone_max);
        self.data[from * self.size + to].store(clamped.to_bits(), Ordering::Relaxed);
    }

    /// Deposit pheromone on edge (from -> to) with MMAS clamping.
    /// Uses CAS loop for lock-free concurrent updates.
    pub fn deposit(&self, from: usize, to: usize, amount: f64) {
        let idx = from * self.size + to;
        let cell = &self.data[idx];
        loop {
            let old = cell.load(Ordering::Relaxed);
            let old_val = f64::from_bits(old);
            let new_val = (old_val + amount).min(self.config.pheromone_max);
            match cell.compare_exchange_weak(
                old,
                new_val.to_bits(),
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(_) => continue, // CAS failed, retry
            }
        }
    }

    /// Evaporate all edges: tau(i,j) = (1 - rho) * tau(i,j).
    /// Clamps to pheromone_min to preserve path diversity.
    pub fn evaporate(&self) {
        self.evaporate_with_rate(self.config.evaporation_rate);
    }

    /// Evaporate with a custom rate (for adaptive evaporation).
    /// v0.3.0: Supports dynamic rho based on latency variance.
    pub fn evaporate_with_rate(&self, rate: f64) {
        let factor = 1.0 - rate;
        let min = self.config.pheromone_min;
        for cell in self.data.iter() {
            loop {
                let old = cell.load(Ordering::Relaxed);
                let old_val = f64::from_bits(old);
                let new_val = (old_val * factor).max(min);
                match cell.compare_exchange_weak(
                    old,
                    new_val.to_bits(),
                    Ordering::AcqRel,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(_) => continue,
                }
            }
        }
    }

    /// Deposit pheromone along an entire path.
    /// deposit_amount = weight / path_cost (inverse proportional to cost).
    pub fn deposit_path(&self, path: &[usize], cost: f64, weight: f64) {
        if path.len() < 2 || cost <= 0.0 {
            return;
        }
        let amount = weight / cost;
        for window in path.windows(2) {
            self.deposit(window[0], window[1], amount);
        }
    }

    /// Reset all pheromone to initial value. Used when topology changes
    /// invalidate the existing trail map.
    pub fn reset(&self) {
        let initial_bits = self.config.initial_pheromone.to_bits();
        for cell in self.data.iter() {
            cell.store(initial_bits, Ordering::Relaxed);
        }
    }

    /// Generate a serializable snapshot for diagnostics.
    pub fn snapshot(&self) -> PheromoneSnapshot {
        let mut edges = Vec::new();
        let mut total = 0.0;
        let mut count = 0usize;

        for i in 0..self.size {
            for j in 0..self.size {
                if i == j {
                    continue;
                }
                let intensity = self.get(i, j);
                if intensity > self.config.pheromone_min * 1.1 {
                    edges.push(PheromoneEdge {
                        from: i,
                        to: j,
                        intensity,
                    });
                }
                total += intensity;
                count += 1;
            }
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        PheromoneSnapshot {
            size: self.size,
            edges,
            total_pheromone: total,
            avg_pheromone: if count > 0 { total / count as f64 } else { 0.0 },
            timestamp_ms: now,
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> AcoConfig {
        AcoConfig {
            initial_pheromone: 1.0,
            evaporation_rate: 0.5,
            pheromone_min: 0.1,
            pheromone_max: 5.0,
            ..AcoConfig::default()
        }
    }

    #[test]
    fn test_initial_pheromone() {
        let matrix = PheromoneMatrix::new(4, &test_config());
        assert_eq!(matrix.get(0, 1), 1.0);
        assert_eq!(matrix.get(2, 3), 1.0);
    }

    #[test]
    fn test_deposit_and_clamp() {
        let matrix = PheromoneMatrix::new(4, &test_config());
        matrix.deposit(0, 1, 3.0);
        assert_eq!(matrix.get(0, 1), 4.0);

        matrix.deposit(0, 1, 10.0);
        assert_eq!(matrix.get(0, 1), 5.0); // clamped to max
    }

    #[test]
    fn test_evaporation() {
        let matrix = PheromoneMatrix::new(4, &test_config());
        matrix.deposit(0, 1, 3.0); // now 4.0
        matrix.evaporate(); // 4.0 * 0.5 = 2.0
        assert!((matrix.get(0, 1) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_evaporation_min_clamp() {
        let config = AcoConfig {
            initial_pheromone: 0.05,
            evaporation_rate: 0.99,
            pheromone_min: 0.1,
            ..AcoConfig::default()
        };
        let matrix = PheromoneMatrix::new(2, &config);
        matrix.evaporate();
        assert_eq!(matrix.get(0, 1), 0.1); // clamped to min
    }

    #[test]
    fn test_deposit_path() {
        let matrix = PheromoneMatrix::new(4, &test_config());
        let path = vec![0, 1, 2, 3];
        matrix.deposit_path(&path, 2.0, 1.0); // deposit 0.5 on each edge
        assert!((matrix.get(0, 1) - 1.5).abs() < 1e-10);
        assert!((matrix.get(1, 2) - 1.5).abs() < 1e-10);
        assert!((matrix.get(2, 3) - 1.5).abs() < 1e-10);
    }

    #[test]
    fn test_reset() {
        let matrix = PheromoneMatrix::new(4, &test_config());
        matrix.deposit(0, 1, 3.0);
        matrix.reset();
        assert_eq!(matrix.get(0, 1), 1.0);
    }

    #[test]
    fn test_snapshot() {
        let matrix = PheromoneMatrix::new(3, &test_config());
        matrix.deposit(0, 1, 2.0);
        let snap = matrix.snapshot();
        assert_eq!(snap.size, 3);
        assert!(snap.total_pheromone > 0.0);
    }

    #[test]
    fn test_evaporate_with_custom_rate() {
        let matrix = PheromoneMatrix::new(4, &test_config());
        matrix.deposit(0, 1, 3.0); // now 4.0
        matrix.evaporate_with_rate(0.75); // 4.0 * 0.25 = 1.0
        assert!((matrix.get(0, 1) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_concurrent_deposits() {
        use std::thread;

        let config = test_config();
        let matrix = Arc::new(PheromoneMatrix::new(4, &config));

        let handles: Vec<_> = (0..8)
            .map(|_| {
                let m = Arc::clone(&matrix);
                thread::spawn(move || {
                    for _ in 0..100 {
                        m.deposit(0, 1, 0.01);
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        // 1.0 initial + 800 * 0.01 = 9.0, clamped to 5.0
        assert_eq!(matrix.get(0, 1), 5.0);
    }
}
