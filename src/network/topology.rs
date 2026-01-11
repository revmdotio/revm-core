use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    EntryPoint,
    Relay,
    Validator,
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

    pub fn add_edge(&mut self, from: usize, to: usize, latency_ms: f64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default().as_millis() as u64;
        if let Some(edge) = self.adjacency[from].iter_mut().find(|e| e.to == to) {
            edge.latency_ms = latency_ms;
            edge.last_measured = now;
            return;
        }
        self.adjacency[from].push(Edge { to, latency_ms, last_measured: now });
    }

    pub fn neighbors(&self, node: usize) -> Vec<usize> {
        self.adjacency[node].iter().map(|e| e.to).collect()
    }

    pub fn edge_latency(&self, from: usize, to: usize) -> f64 {
        self.adjacency[from].iter().find(|e| e.to == to)
            .map(|e| e.latency_ms).unwrap_or(f64::MAX)
    }

    pub fn node_count(&self) -> usize { self.num_nodes }
    pub fn edge_count(&self) -> usize { self.adjacency.iter().map(|e| e.len()).sum() }

    pub fn update_latency(&mut self, from: usize, to: usize, latency_ms: f64) {
        self.add_edge(from, to, latency_ms);
    }

    pub fn add_edge_bidirectional(&mut self, a: usize, b: usize, latency_ms: f64) {
        self.add_edge(a, b, latency_ms);
        self.add_edge(b, a, latency_ms);
    }

    pub fn set_node_info(&mut self, id: usize, label: String, node_type: NodeType) {
        self.node_labels.insert(id, NodeInfo { id, label, node_type, stake_weight: None });
    }
}

