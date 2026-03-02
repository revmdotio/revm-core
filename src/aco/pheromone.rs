use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::config::AcoConfig;

/// Thread-safe pheromone matrix for the colony graph.
///
/// Stores pheromone intensity on each directed edge (i -> j).
/// Uses MMAS (Max-Min Ant System) bounds to prevent stagnation.
#[derive(Debug, Clone)]
pub struct PheromoneMatrix {
    size: usize,
    data: Arc<RwLock<Vec<f64>>>,
    config: AcoConfig,
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
        let data = vec![config.initial_pheromone; size * size];
        Self {
            size,
            data: Arc::new(RwLock::new(data)),
            config: config.clone(),
        }
    }

    /// Get pheromone intensity on edge (from -> to).
    pub fn get(&self, from: usize, to: usize) -> f64 {
        let data = self.data.read();
        data[from * self.size + to]
    }

    /// Deposit pheromone on edge (from -> to) with MMAS clamping.
    pub fn deposit(&self, from: usize, to: usize, amount: f64) {
        let mut data = self.data.write();
        let idx = from * self.size + to;
        let new_val = (data[idx] + amount).min(self.config.pheromone_max);
        data[idx] = new_val;
    }

    /// Evaporate all edges: tau(i,j) = (1 - rho) * tau(i,j).
    /// Clamps to pheromone_min to preserve path diversity.
    pub fn evaporate(&self) {
        let mut data = self.data.write();
        let factor = 1.0 - self.config.evaporation_rate;
        for val in data.iter_mut() {
            *val = (*val * factor).max(self.config.pheromone_min);
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
        let mut data = self.data.write();
        for val in data.iter_mut() {
            *val = self.config.initial_pheromone;
        }
    }

    /// Generate a serializable snapshot for diagnostics.
    pub fn snapshot(&self) -> PheromoneSnapshot {
        let data = self.data.read();
        let mut edges = Vec::new();
        let mut total = 0.0;
        let mut count = 0usize;

        for i in 0..self.size {
            for j in 0..self.size {
                if i == j {
                    continue;
                }
                let intensity = data[i * self.size + j];
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
}
