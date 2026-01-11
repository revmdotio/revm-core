pub mod aco;
pub mod network;

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

    #[error("Configuration invalid: {0}")]
    ConfigError(String),
}

pub type Result<T> = std::result::Result<T, RevmError>;
