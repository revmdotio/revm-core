use serde::{Deserialize, Serialize};

/// Core parameters for the Ant Colony Optimization engine.
///
/// Based on Dorigo 1996 (Ant System) and Di Caro & Dorigo 1998 (AntNet),
/// tuned for sub-10ms Solana transaction routing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcoConfig {
    /// Pheromone influence factor (alpha). Higher values make ants follow
    /// existing trails more aggressively.
    pub alpha: f64,

    /// Heuristic influence factor (beta). Higher values make ants prefer
    /// shorter/faster edges based on latency measurements.
    pub beta: f64,

    /// Evaporation rate (rho). Controls how fast old pheromone decays.
    /// Range: (0.0, 1.0). Solana slot times (~400ms) require aggressive
    /// evaporation to adapt to leader rotation.
    pub evaporation_rate: f64,

    /// Initial pheromone deposit on all edges. Set high enough to encourage
    /// exploration in early iterations.
    pub initial_pheromone: f64,

    /// Minimum pheromone threshold. Prevents complete trail extinction on
    /// any edge, preserving path diversity.
    pub pheromone_min: f64,

    /// Maximum pheromone cap. Prevents runaway convergence on a single path
    /// (MMAS bounds from Stutzle & Hoos 2000).
    pub pheromone_max: f64,

    /// Number of ants dispatched per routing cycle.
    pub ant_count: usize,

    /// Maximum iterations before declaring no path found.
    pub max_iterations: u32,

    /// Pheromone deposit weight for the global-best ant.
    /// Only the best ant deposits pheromone each iteration (ACS strategy).
    pub deposit_weight: f64,

    /// Latency weight in heuristic calculation.
    /// heuristic(edge) = latency_weight / measured_latency_ms
    pub latency_weight: f64,

    /// Stale threshold in milliseconds. Edges with latency measurements
    /// older than this value are re-probed before routing.
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
            return Err(crate::RevmError::ConfigError(
                "alpha must be non-negative".into(),
            ));
        }
        if self.beta < 0.0 {
            return Err(crate::RevmError::ConfigError(
                "beta must be non-negative".into(),
            ));
        }
        if self.evaporation_rate <= 0.0 || self.evaporation_rate >= 1.0 {
            return Err(crate::RevmError::ConfigError(
                "evaporation_rate must be in (0.0, 1.0)".into(),
            ));
        }
        if self.pheromone_min >= self.pheromone_max {
            return Err(crate::RevmError::ConfigError(
                "pheromone_min must be less than pheromone_max".into(),
            ));
        }
        if self.ant_count == 0 {
            return Err(crate::RevmError::ConfigError(
                "ant_count must be at least 1".into(),
            ));
        }
        Ok(())
    }

    /// High-frequency config tuned for Solana mainnet conditions.
    /// Aggressive evaporation (0.25) to track leader rotation within ~2 slots.
    pub fn mainnet() -> Self {
        Self {
            alpha: 1.2,
            beta: 3.0,
            evaporation_rate: 0.25,
            initial_pheromone: 0.05,
            pheromone_min: 0.001,
            pheromone_max: 8.0,
            ant_count: 64,
            max_iterations: 50,
            deposit_weight: 1.5,
            latency_weight: 1.2,
            stale_threshold_ms: 1200,
            
        }
    }

    /// Conservative config for devnet/testnet with relaxed timing.
    pub fn devnet() -> Self {
        Self {
            alpha: 1.0,
            beta: 2.0,
            evaporation_rate: 0.10,
            initial_pheromone: 0.2,
            pheromone_min: 0.01,
            pheromone_max: 15.0,
            ant_count: 16,
            max_iterations: 200,
            deposit_weight: 1.0,
            latency_weight: 0.8,
            stale_threshold_ms: 5000,
        }
    }
}
