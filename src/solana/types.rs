use serde::{Deserialize, Serialize};

/// Raw transaction payload ready for submission to Solana.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionPayload {
    /// Base58-encoded serialized transaction
    pub data: String,
    /// Priority fee in microlamports (for compute budget)
    pub priority_fee: Option<u64>,
    /// Whether to skip preflight simulation
    pub skip_preflight: bool,
    /// Maximum number of retries before giving up
    pub max_retries: Option<u8>,
}

/// Result of a transaction send attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendResult {
    /// Transaction signature (Base58)
    pub signature: String,
    /// Which validator received the transaction
    pub target_validator: String,
    /// Measured send latency in milliseconds
    pub send_latency_ms: f64,
    /// Number of hops in the ACO-optimized path
    pub hop_count: usize,
    /// Slot at time of submission
    pub slot: u64,
    /// Whether the transaction was confirmed
    pub confirmed: bool,
}

/// Solana cluster endpoint configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    pub rpc_url: String,
    pub ws_url: Option<String>,
    pub tpu_proxy_addr: Option<String>,
    pub commitment: CommitmentLevel,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CommitmentLevel {
    Processed,
    Confirmed,
    Finalized,
}

impl Default for CommitmentLevel {
    fn default() -> Self {
        Self::Confirmed
    }
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            rpc_url: "https://api.mainnet-beta.solana.com".into(),
            ws_url: Some("wss://api.mainnet-beta.solana.com".into()),
            tpu_proxy_addr: None,
            commitment: CommitmentLevel::Confirmed,
        }
    }
}

impl ClusterConfig {
    pub fn devnet() -> Self {
        Self {
            rpc_url: "https://api.devnet.solana.com".into(),
            ws_url: Some("wss://api.devnet.solana.com".into()),
            tpu_proxy_addr: None,
            commitment: CommitmentLevel::Confirmed,
        }
    }
}
