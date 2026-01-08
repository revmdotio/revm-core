use serde::{Deserialize, Serialize};

/// Core parameters for the Ant Colony Optimization engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcoConfig {
    pub alpha: f64,
    pub beta: f64,
    pub evaporation_rate: f64,
    pub initial_pheromone: f64,
    pub pheromone_min: f64,
    pub pheromone_max: f64,
    pub ant_count: usize,
    pub max_iterations: u32,
    pub deposit_weight: f64,
    pub latency_weight: f64,
    pub stale_threshold_ms: u64,
}

impl Default for AcoConfig {
    fn default() -> Self {
        Self {
            alpha: 1.0,
            beta: 2.5,
            evaporation_rate: 0.15,
            initial_pheromone: 0.1,
            pheromone_min: 0.001,
            pheromone_max: 10.0,
            ant_count: 32,
            max_iterations: 100,
            deposit_weight: 1.0,
            latency_weight: 1.0,
            stale_threshold_ms: 2000,
        }
    }
}

impl AcoConfig {
    pub fn validate(&self) -> crate::Result<()> {
        if self.alpha < 0.0 {
            return Err(crate::RevmError::ConfigError("alpha must be non-negative".into()));
        }
        if self.evaporation_rate <= 0.0 || self.evaporation_rate >= 1.0 {
            return Err(crate::RevmError::ConfigError("evaporation_rate must be in (0.0, 1.0)".into()));
        }
        if self.pheromone_min >= self.pheromone_max {
            return Err(crate::RevmError::ConfigError("pheromone_min must be less than pheromone_max".into()));
        }
        if self.ant_count == 0 {
            return Err(crate::RevmError::ConfigError("ant_count must be at least 1".into()));
        }
        Ok(())
    }
}
