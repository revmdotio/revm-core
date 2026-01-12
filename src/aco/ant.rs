use rand::Rng;

use super::config::AcoConfig;
use super::pheromone::PheromoneMatrix;
use crate::network::topology::NetworkTopology;

/// A single ant agent that constructs a path through the network graph.
#[derive(Debug)]
pub struct Ant {
    pub path: Vec<usize>,
    pub cost: f64,
    visited: Vec<bool>,
}

impl Ant {
    pub fn new(num_nodes: usize) -> Self {
        Self {
            path: Vec::with_capacity(num_nodes),
            cost: 0.0,
            visited: vec![false; num_nodes],
        }
    }

    pub fn path_length(&self) -> usize {
        self.path.len()
    }

    pub fn hop_count(&self) -> usize {
        if self.path.is_empty() { 0 } else { self.path.len() - 1 }
    }
}
