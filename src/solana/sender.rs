use log::{info, warn};
use std::time::Instant;

use super::types::{ClusterConfig, CommitmentLevel, SendResult, TransactionPayload};
use crate::router::engine::{RouteDecision, RoutingEngine};
use crate::Result;

/// Sends transactions to Solana validators using ACO-optimized routing.
///
/// Integrates with the RoutingEngine to select the optimal validator target,
/// then submits the transaction via TPU (QUIC) or RPC fallback.
pub struct TransactionSender {
    cluster: ClusterConfig,
    engine: RoutingEngine,
    use_tpu: bool,
}

impl TransactionSender {
    pub fn new(cluster: ClusterConfig, engine: RoutingEngine) -> Self {
        Self {
            cluster,
            engine,
            use_tpu: true,
        }
    }

    /// Disable TPU direct send, fall back to RPC sendTransaction.
    pub fn disable_tpu(&mut self) {
        self.use_tpu = false;
    }

    /// Send a transaction using ACO-optimized routing.
    ///
    /// Flow:
    /// 1. Engine selects optimal path via ACO
    /// 2. Extract target validator from path
    /// 3. Submit via TPU/QUIC (preferred) or RPC fallback
    /// 4. Feed latency measurement back into the colony
    pub async fn send(&mut self, payload: &TransactionPayload) -> Result<SendResult> {
        let start = Instant::now();

        // Step 1: Get routing decision
        let decision = self.engine.route_transaction(0)?;

        // Step 2: Extract target
        let target_node = *decision.primary_path.last().unwrap_or(&0);
        let target_label = self
            .engine
            .colony()
            .topology()
            .node_info(target_node)
            .map(|n| n.label.clone())
            .unwrap_or_else(|| "unknown".into());

        // Step 3: Submit transaction
        let signature = if self.use_tpu {
            self.send_via_tpu(payload, &target_label).await?
        } else {
            self.send_via_rpc(payload).await?
        };

        let send_latency = start.elapsed().as_secs_f64() * 1000.0;

        // Step 4: Feed measurement back to colony
        if decision.primary_path.len() >= 2 {
            let from = decision.primary_path[0];
            let to = decision.primary_path[1];
            self.engine
                .colony_mut()
                .update_edge_latency(from, to, send_latency);
        }

        info!(
            "Transaction sent: sig={}, target={}, latency={:.1}ms",
            &signature[..8],
            target_label,
            send_latency
        );

        Ok(SendResult {
            signature,
            target_validator: target_label,
            send_latency_ms: send_latency,
            hop_count: decision.hop_count,
            slot: 0, // populated by confirmation
            confirmed: false,
        })
    }

    /// Submit transaction directly to validator's TPU port via QUIC.
    async fn send_via_tpu(
        &self,
        payload: &TransactionPayload,
        _target_pubkey: &str,
    ) -> Result<String> {
        // In production, this would:
        // 1. Resolve the validator's TPU QUIC address from cluster info
        // 2. Establish QUIC connection with stake-weighted priority
        // 3. Send the serialized transaction bytes
        //
        // For now, falls back to RPC
        warn!("TPU/QUIC send pending implementation, falling back to RPC");
        self.send_via_rpc(payload).await
    }

    /// Submit transaction via JSON-RPC sendTransaction.
    async fn send_via_rpc(&self, payload: &TransactionPayload) -> Result<String> {
        let commitment = match self.cluster.commitment {
            CommitmentLevel::Processed => "processed",
            CommitmentLevel::Confirmed => "confirmed",
            CommitmentLevel::Finalized => "finalized",
        };

        let params = serde_json::json!([
            payload.data,
            {
                "skipPreflight": payload.skip_preflight,
                "preflightCommitment": commitment,
                "maxRetries": payload.max_retries.unwrap_or(3),
                "encoding": "base58"
            }
        ]);

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendTransaction",
            "params": params
        });

        let client = reqwest::Client::new();
        let response = client
            .post(&self.cluster.rpc_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| crate::RevmError::RpcError(e.to_string()))?;

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| crate::RevmError::RpcError(e.to_string()))?;

        if let Some(error) = result.get("error") {
            return Err(crate::RevmError::SimulationError(error.to_string()));
        }

        result["result"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| crate::RevmError::RpcError("Missing signature in response".into()))
    }

    pub fn engine(&self) -> &RoutingEngine {
        &self.engine
    }

    pub fn engine_mut(&mut self) -> &mut RoutingEngine {
        &mut self.engine
    }
}
