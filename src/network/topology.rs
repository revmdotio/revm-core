use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Directed weighted graph representing the validator network topology.
///
/// Nodes are RPC/TPU endpoints (validators, relays, entry points).
/// Edge weights are measured latency in milliseconds.
#[derive(Debug, Clone)]
pub struct NetworkTopology {
    num_nodes: usize,
    adjacency: Vec<Vec<Edge>>,
    node_labels: HashMap<usize, NodeInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub to: usize,
    pub latency_ms: f64,
    pub last_measured: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: usize,
    pub label: String,
    pub node_type: NodeType,
    pub stake_weight: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum NodeType {
    /// Client entry point (user's RPC endpoint)
    EntryPoint,
    /// RPC relay node
    Relay,
    /// Validator with TPU access
    Validator,
    /// Staked validator currently in leader schedule
    LeaderValidator,
}

impl NetworkTopology {
    pub fn new(num_nodes: usize) -> Self {
        Self {
            num_nodes,
            adjacency: vec![Vec::new(); num_nodes],
            node_labels: HashMap::new(),
        }
    }

    /// Add a directed edge from `from` to `to` with measured latency.
    pub fn add_edge(&mut self, from: usize, to: usize, latency_ms: f64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // Update existing edge if present
        if let Some(edge) = self.adjacency[from].iter_mut().find(|e| e.to == to) {
            edge.latency_ms = latency_ms;
            edge.last_measured = now;
            return;
        }

        self.adjacency[from].push(Edge {
            to,
            latency_ms,
            last_measured: now,
        });
    }

    /// Add bidirectional edge.
    pub fn add_edge_bidirectional(&mut self, a: usize, b: usize, latency_ms: f64) {
        self.add_edge(a, b, latency_ms);
        self.add_edge(b, a, latency_ms);
    }

    /// Set node metadata.
    pub fn set_node_info(&mut self, id: usize, label: String, node_type: NodeType) {
        self.node_labels.insert(
            id,
            NodeInfo {
                id,
                label,
                node_type,
                stake_weight: None,
            },
        );
    }

    /// Set node metadata with stake weight.
    pub fn set_node_info_with_stake(
        &mut self,
        id: usize,
        label: String,
        node_type: NodeType,
        stake_weight: f64,
    ) {
        self.node_labels.insert(
            id,
            NodeInfo {
                id,
                label,
                node_type,
                stake_weight: Some(stake_weight),
            },
        );
    }

    /// Get neighbors of a node (outgoing edges).
    pub fn neighbors(&self, node: usize) -> Vec<usize> {
        self.adjacency[node].iter().map(|e| e.to).collect()
    }

    /// Get latency on edge (from -> to). Returns f64::MAX if edge doesn't exist.
    pub fn edge_latency(&self, from: usize, to: usize) -> f64 {
        self.adjacency[from]
            .iter()
            .find(|e| e.to == to)
            .map(|e| e.latency_ms)
            .unwrap_or(f64::MAX)
    }

    /// Update latency measurement on an existing edge.
    pub fn update_latency(&mut self, from: usize, to: usize, latency_ms: f64) {
        self.add_edge(from, to, latency_ms);
    }

    /// Get all edges with measurements older than `threshold_ms`.
    pub fn stale_edges(&self, threshold_ms: u64) -> Vec<(usize, usize)> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let mut stale = Vec::new();
        for (from, edges) in self.adjacency.iter().enumerate() {
            for edge in edges {
                if now - edge.last_measured > threshold_ms {
                    stale.push((from, edge.to));
                }
            }
        }
        stale
    }

    /// Find all validators currently marked as leaders.
    pub fn leader_validators(&self) -> Vec<usize> {
        self.node_labels
            .values()
            .filter(|n| n.node_type == NodeType::LeaderValidator)
            .map(|n| n.id)
            .collect()
    }

    /// Promote a validator to leader status (called when leader schedule updates).
    pub fn promote_to_leader(&mut self, node_id: usize) {
        if let Some(info) = self.node_labels.get_mut(&node_id) {
            info.node_type = NodeType::LeaderValidator;
        }
    }

    /// Demote all leaders back to regular validators (called at epoch boundary).
    pub fn demote_all_leaders(&mut self) {
        for info in self.node_labels.values_mut() {
            if info.node_type == NodeType::LeaderValidator {
                info.node_type = NodeType::Validator;
            }
        }
    }

    pub fn node_count(&self) -> usize {
        self.num_nodes
    }

    pub fn edge_count(&self) -> usize {
        self.adjacency.iter().map(|edges| edges.len()).sum()
    }

    pub fn node_info(&self, id: usize) -> Option<&NodeInfo> {
        self.node_labels.get(&id)
    }

    /// Build a topology from a Solana cluster snapshot.
    /// Maps validators to graph nodes weighted by stake and measured latency.
    pub fn from_cluster_snapshot(
        validators: Vec<ValidatorEntry>,
        entry_point: &str,
    ) -> Self {
        let num_nodes = validators.len() + 1; // +1 for entry point
        let mut topo = Self::new(num_nodes);

        // Node 0 is always the entry point
        topo.set_node_info(0, entry_point.to_string(), NodeType::EntryPoint);

        for (i, v) in validators.iter().enumerate() {
            let node_id = i + 1;
            let node_type = if v.is_leader {
                NodeType::LeaderValidator
            } else {
                NodeType::Validator
            };

            topo.set_node_info_with_stake(
                node_id,
                v.pubkey.clone(),
                node_type,
                v.stake_weight,
            );

            // Entry point -> validator edge (initial estimate)
            topo.add_edge(0, node_id, v.estimated_latency_ms);

            // Validator -> validator edges (mesh connectivity)
            for (j, v2) in validators.iter().enumerate() {
                if i != j {
                    let inter_latency = (v.estimated_latency_ms + v2.estimated_latency_ms) * 0.3;
                    topo.add_edge(node_id, j + 1, inter_latency);
                }
            }
        }

        topo
    }
}

/// Validator entry from cluster info, used to build initial topology.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorEntry {
    pub pubkey: String,
    pub stake_weight: f64,
    pub estimated_latency_ms: f64,
    pub is_leader: bool,
    pub tpu_addr: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_query_edges() {
        let mut topo = NetworkTopology::new(4);
        topo.add_edge(0, 1, 5.0);
        topo.add_edge(0, 2, 3.0);
        topo.add_edge(1, 3, 2.0);

        assert_eq!(topo.neighbors(0), vec![1, 2]);
        assert_eq!(topo.edge_latency(0, 1), 5.0);
        assert_eq!(topo.edge_latency(0, 2), 3.0);
        assert_eq!(topo.edge_latency(0, 3), f64::MAX); // no direct edge
    }

    #[test]
    fn test_bidirectional() {
        let mut topo = NetworkTopology::new(3);
        topo.add_edge_bidirectional(0, 1, 4.0);

        assert_eq!(topo.edge_latency(0, 1), 4.0);
        assert_eq!(topo.edge_latency(1, 0), 4.0);
    }

    #[test]
    fn test_update_latency() {
        let mut topo = NetworkTopology::new(3);
        topo.add_edge(0, 1, 10.0);
        assert_eq!(topo.edge_latency(0, 1), 10.0);

        topo.update_latency(0, 1, 3.0);
        assert_eq!(topo.edge_latency(0, 1), 3.0);
    }

    #[test]
    fn test_leader_management() {
        let mut topo = NetworkTopology::new(3);
        topo.set_node_info(0, "entry".into(), NodeType::EntryPoint);
        topo.set_node_info(1, "val1".into(), NodeType::Validator);
        topo.set_node_info(2, "val2".into(), NodeType::Validator);

        topo.promote_to_leader(1);
        assert_eq!(topo.leader_validators(), vec![1]);

        topo.demote_all_leaders();
        assert!(topo.leader_validators().is_empty());
    }

    #[test]
    fn test_from_cluster_snapshot() {
        let validators = vec![
            ValidatorEntry {
                pubkey: "Va1id1111111111111111111111111111111111111111".into(),
                stake_weight: 0.05,
                estimated_latency_ms: 8.0,
                is_leader: true,
                tpu_addr: Some("127.0.0.1:8004".into()),
            },
            ValidatorEntry {
                pubkey: "Va1id2222222222222222222222222222222222222222".into(),
                stake_weight: 0.03,
                estimated_latency_ms: 12.0,
                is_leader: false,
                tpu_addr: None,
            },
        ];

        let topo = NetworkTopology::from_cluster_snapshot(validators, "rpc.mainnet.solana.com");
        assert_eq!(topo.node_count(), 3);
        assert!(topo.edge_latency(0, 1) < 100.0);
    }
}
