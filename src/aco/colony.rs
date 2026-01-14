use log::{debug, info};
use serde::{Deserialize, Serialize};

use super::ant::Ant;
use super::config::AcoConfig;
use super::pheromone::PheromoneMatrix;
use crate::network::topology::NetworkTopology;
use crate::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingResult {
    pub path: Vec<usize>,
    pub cost: f64,
    pub hop_count: usize,
    pub iterations_used: u32,
    pub ants_dispatched: usize,
}

pub struct Colony {
    config: AcoConfig,
    pheromone: PheromoneMatrix,
    topology: NetworkTopology,
    best_path: Option<Vec<usize>>,
    best_cost: f64,
    total_routes: u64,
}

impl Colony {
    pub fn new(topology: NetworkTopology, config: AcoConfig) -> Result<Self> {
        config.validate()?;
        let size = topology.node_count();
        let pheromone = PheromoneMatrix::new(size, &config);
        Ok(Self {
            config, pheromone, topology,
            best_path: None, best_cost: f64::MAX, total_routes: 0,
        })
    }

    pub fn best_path(&self) -> Option<&Vec<usize>> { self.best_path.as_ref() }
    pub fn best_cost(&self) -> f64 { self.best_cost }
    pub fn total_routes(&self) -> u64 { self.total_routes }
    pub fn pheromone(&self) -> &PheromoneMatrix { &self.pheromone }
    pub fn topology(&self) -> &NetworkTopology { &self.topology }
}
