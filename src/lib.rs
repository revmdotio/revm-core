pub mod aco;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RevmError {
    #[error("Colony not initialized")]
    ColonyNotInitialized,

    #[error("No viable path found after {0} iterations")]
    NoPathFound(u32),

    #[error("Node {0} not found in topology")]
    NodeNotFound(String),

    #[error("Configuration invalid: {0}")]
    ConfigError(String),
}

pub type Result<T> = std::result::Result<T, RevmError>;
