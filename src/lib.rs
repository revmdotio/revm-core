pub mod aco;
pub mod network;
pub mod router;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RevmError {
    #[error("Colony not initialized: call Colony::new() first")]
    ColonyNotInitialized,

    #[error("No viable path found after {0} iterations")]
    NoPathFound(u32),

    #[error("Pheromone matrix overflow at edge ({0}, {1})")]
    PheromoneOverflow(usize, usize),

    #[error("Node {0} not found in topology")]
    NodeNotFound(String),

    #[error("RPC connection failed: {0}")]
    RpcError(String),

    #[error("Transaction simulation failed: {0}")]
    SimulationError(String),

    #[error("Leader schedule unavailable for slot {0}")]
    LeaderScheduleError(u64),

    #[error("TPU connection refused by validator {0}")]
    TpuConnectionError(String),

    #[error("Configuration invalid: {0}")]
    ConfigError(String),
}

pub type Result<T> = std::result::Result<T, RevmError>;
