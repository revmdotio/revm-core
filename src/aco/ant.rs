use rand::Rng;

use super::config::AcoConfig;
use super::pheromone::PheromoneMatrix;
use crate::network::topology::NetworkTopology;

/// A single ant agent that constructs a path through the network graph.
///
/// Each ant starts at a source node and probabilistically selects next hops
/// based on pheromone intensity and edge heuristic (inverse latency).
/// Implements the state transition rule from ACS (Ant Colony System).
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

    /// Construct a path from `source` to `destination` using ACO probability rules.
    ///
    /// At each node, the ant selects the next hop with probability:
    ///   p(i,j) = [tau(i,j)^alpha * eta(i,j)^beta] / sum_k[tau(i,k)^alpha * eta(i,k)^beta]
    ///
    /// where tau is pheromone and eta = latency_weight / latency(i,j).
    pub fn find_path(
        &mut self,
        source: usize,
        destination: usize,
        pheromone: &PheromoneMatrix,
        topology: &NetworkTopology,
        config: &AcoConfig,
    ) -> bool {
        self.path.clear();
        self.cost = 0.0;
        for v in self.visited.iter_mut() {
            *v = false;
        }

        self.path.push(source);
        self.visited[source] = true;

        let mut current = source;
        let mut rng = rand::thread_rng();

        while current != destination {
            let neighbors = topology.neighbors(current);
            let candidates: Vec<usize> = neighbors
                .iter()
                .copied()
                .filter(|&n| !self.visited[n])
                .collect();

            if candidates.is_empty() {
                return false; // dead end
            }

            // Calculate selection probabilities
            let mut probabilities: Vec<f64> = Vec::with_capacity(candidates.len());
            let mut total = 0.0;

            for &next in &candidates {
                let tau = pheromone.get(current, next).powf(config.alpha);
                let latency = topology.edge_latency(current, next);
                let eta = if latency > 0.0 {
                    (config.latency_weight / latency).powf(config.beta)
                } else {
                    1.0
                };
                let score = tau * eta;
                probabilities.push(score);
                total += score;
            }

            if total <= 0.0 {
                return false;
            }

            // Roulette wheel selection
            let threshold = rng.gen::<f64>() * total;
            let mut cumulative = 0.0;
            let mut selected = candidates[0];

            for (i, &candidate) in candidates.iter().enumerate() {
                cumulative += probabilities[i];
                if cumulative >= threshold {
                    selected = candidate;
                    break;
                }
            }

            let edge_cost = topology.edge_latency(current, selected);
            self.cost += edge_cost;
            self.path.push(selected);
            self.visited[selected] = true;
            current = selected;
        }

        true
    }

    pub fn path_length(&self) -> usize {
        self.path.len()
    }

    pub fn hop_count(&self) -> usize {
        if self.path.is_empty() {
            0
        } else {
            self.path.len() - 1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ant_direct_path() {
        // Build a simple 3-node linear topology: 0 -> 1 -> 2
        let mut topo = NetworkTopology::new(3);
        topo.add_edge(0, 1, 3.0);
        topo.add_edge(1, 2, 4.0);

        let config = AcoConfig::default();
        let pheromone = PheromoneMatrix::new(3, &config);

        let mut ant = Ant::new(3);
        let found = ant.find_path(0, 2, &pheromone, &topo, &config);

        assert!(found);
        assert_eq!(ant.path, vec![0, 1, 2]);
        assert!((ant.cost - 7.0).abs() < 1e-10);
    }

    #[test]
    fn test_ant_no_path() {
        // Disconnected graph: 0 -> 1, but no edge to 2
        let mut topo = NetworkTopology::new(3);
        topo.add_edge(0, 1, 1.0);

        let config = AcoConfig::default();
        let pheromone = PheromoneMatrix::new(3, &config);

        let mut ant = Ant::new(3);
        let found = ant.find_path(0, 2, &pheromone, &topo, &config);

        assert!(!found);
    }

    #[test]
    fn test_hop_count() {
        let mut ant = Ant::new(5);
        ant.path = vec![0, 3, 4, 2];
        assert_eq!(ant.hop_count(), 3);
    }
}
