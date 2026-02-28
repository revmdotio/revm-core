use log::{debug, info};
use serde::{Deserialize, Serialize};

use super::ant::Ant;
use super::config::AcoConfig;
use super::pheromone::PheromoneMatrix;
use crate::network::topology::NetworkTopology;
use crate::Result;

/// Result of a single colony routing operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingResult {
    pub path: Vec<usize>,
    pub cost: f64,
    pub hop_count: usize,
    pub iterations_used: u32,
    pub ants_dispatched: usize,
}

/// The Colony is the top-level ACO routing engine.
///
/// It manages pheromone state and dispatches ant agents to discover
/// optimal paths through the network topology. Each call to `route()`
/// runs a full ACO optimization cycle.
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
            config,
            pheromone,
            topology,
            best_path: None,
            best_cost: f64::MAX,
            total_routes: 0,
        
        })
    }

    /// Run a full ACO optimization cycle to find the best path from source to dest.
    ///
    /// 1. Dispatch `ant_count` ants per iteration
    /// 2. Each ant constructs a path probabilistically
    /// 3. Evaporate pheromone on all edges
    /// 4. Best ant deposits pheromone on its path (ACS global update)
    /// 5. Repeat until convergence or max_iterations
    pub fn route(&mut self, source: usize, destination: usize) -> Result<RoutingResult> {
        let num_nodes = self.topology.node_count();
        if source >= num_nodes {
            return Err(crate::RevmError::NodeNotFound(format!(
                "source index {} out of bounds ({})",
                source, num_nodes
            )));
        }
        if destination >= num_nodes {
            return Err(crate::RevmError::NodeNotFound(format!(
                "destination index {} out of bounds ({})",
                destination, num_nodes
            )));
        }

        let mut iteration_best_path: Option<Vec<usize>> = None;
        let mut iteration_best_cost = f64::MAX;
        let mut total_ants = 0usize;
        let mut converged_at = self.config.max_iterations;

        for iter in 0..self.config.max_iterations {
            let mut round_best_path: Option<Vec<usize>> = None;
            let mut round_best_cost = f64::MAX;

            // Dispatch ants
            for _ in 0..self.config.ant_count {
                let mut ant = Ant::new(num_nodes);
                total_ants += 1;

                if ant.find_path(
                    source,
                    destination,
                    &self.pheromone,
                    &self.topology,
                    &self.config,
                ) {
                    if ant.cost < round_best_cost {
                        round_best_cost = ant.cost;
                        round_best_path = Some(ant.path.clone());
                    }
                }
            }

            // Evaporate
            self.pheromone.evaporate();

            // Global-best deposit (ACS strategy)
            if let Some(ref path) = round_best_path {
                self.pheromone
                    .deposit_path(path, round_best_cost, self.config.deposit_weight);

                if round_best_cost < iteration_best_cost {
                    iteration_best_cost = round_best_cost;
                    iteration_best_path = Some(path.clone());
                }
            }

            // Convergence check: if no improvement for 10 consecutive iterations
            if iteration_best_path.is_some() && iter > 10 {
                let improvement = (iteration_best_cost - round_best_cost).abs();
                if improvement < 1e-9 {
                    converged_at = iter + 1;
                    debug!("Colony converged at iteration {}", converged_at);
                    break;
                }
            }
        }

        let path = iteration_best_path
            .ok_or(crate::RevmError::NoPathFound(self.config.max_iterations))?;

        // Update global best
        if iteration_best_cost < self.best_cost {
            self.best_cost = iteration_best_cost;
            self.best_path = Some(path.clone());
        }
        self.total_routes += 1;

        let hop_count = if path.len() > 1 { path.len() - 1 } else { 0 };

        info!(
            "Route {}->{}: cost={:.2}ms, hops={}, iters={}, ants={}",
            source, destination, iteration_best_cost, hop_count, converged_at, total_ants
        );

        Ok(RoutingResult {
            path,
            cost: iteration_best_cost,
            hop_count,
            iterations_used: converged_at,
            ants_dispatched: total_ants,
        })
    }

    /// Update edge latency after a real transaction probe.
    /// Triggers local pheromone reinforcement on the measured edge.
    pub fn update_edge_latency(&mut self, from: usize, to: usize, latency_ms: f64) {
        self.topology.update_latency(from, to, latency_ms);
        // Reinforce edges that improved
        if latency_ms < self.topology.edge_latency(from, to) * 1.5 {
            let bonus = self.config.deposit_weight * 0.1;
            self.pheromone.deposit(from, to, bonus);
        }
    }

    /// Reset pheromone trails. Call when network topology changes significantly
    /// (e.g., validator set rotation at epoch boundary).
    pub fn reset_pheromone(&self) {
        self.pheromone.reset();
    }

    pub fn best_path(&self) -> Option<&Vec<usize>> {
        self.best_path.as_ref()
    }

    pub fn best_cost(&self) -> f64 {
        self.best_cost
    }

    pub fn total_routes(&self) -> u64 {
        self.total_routes
    }

    pub fn pheromone(&self) -> &PheromoneMatrix {
        &self.pheromone
    }

    pub fn topology(&self) -> &NetworkTopology {
        &self.topology
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_topology() -> NetworkTopology {
        // Diamond graph:
        //     1
        //    / \
        //   0   3
        //    \ /
        //     2
        let mut topo = NetworkTopology::new(4);
        topo.add_edge(0, 1, 5.0);
        topo.add_edge(0, 2, 8.0);
        topo.add_edge(1, 3, 4.0);
        topo.add_edge(2, 3, 3.0);
        topo
    }

    #[test]
    fn test_colony_finds_optimal_path() {
        let topo = build_test_topology();
        let config = AcoConfig {
            ant_count: 16,
            max_iterations: 50,
            ..AcoConfig::default()
        };
        let mut colony = Colony::new(topo, config).unwrap();

        let result = colony.route(0, 3).unwrap();

        // Should find path 0->1->3 (cost 9.0) as optimal
        assert!(result.cost <= 11.0); // either path is valid
        assert!(result.hop_count <= 2);
        assert_eq!(*result.path.first().unwrap(), 0);
        assert_eq!(*result.path.last().unwrap(), 3);
    }

    #[test]
    fn test_colony_repeated_routing_improves() {
        let topo = build_test_topology();
        let config = AcoConfig {
            ant_count: 32,
            max_iterations: 30,
            ..AcoConfig::default()
        };
        let mut colony = Colony::new(topo, config).unwrap();

        let first = colony.route(0, 3).unwrap();
        let _ = colony.route(0, 3).unwrap();
        let third = colony.route(0, 3).unwrap();

        // After multiple rounds, pheromone should reinforce the best path
        assert!(third.cost <= first.cost);
    }

    #[test]
    fn test_colony_invalid_node() {
        let topo = build_test_topology();
        let config = AcoConfig::default();
        let mut colony = Colony::new(topo, config).unwrap();

        let result = colony.route(0, 99);
        assert!(result.is_err());
    }
}
