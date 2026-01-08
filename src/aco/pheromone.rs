use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::config::AcoConfig;

/// Thread-safe pheromone matrix for the colony graph.
#[derive(Debug, Clone)]
pub struct PheromoneMatrix {
    size: usize,
    data: Arc<RwLock<Vec<f64>>>,
    config: AcoConfig,
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

    pub fn get(&self, from: usize, to: usize) -> f64 {
        let data = self.data.read();
        data[from * self.size + to]
    }

    pub fn deposit(&self, from: usize, to: usize, amount: f64) {
        let mut data = self.data.write();
        let idx = from * self.size + to;
        let new_val = (data[idx] + amount).min(self.config.pheromone_max);
        data[idx] = new_val;
    }

    pub fn evaporate(&self) {
        let mut data = self.data.write();
        let factor = 1.0 - self.config.evaporation_rate;
        for val in data.iter_mut() {
            *val = (*val * factor).max(self.config.pheromone_min);
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }
}
