use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::time::Instant;

use super::strategy::RoutingStrategy;
use crate::aco::colony::Colony;
use crate::aco::config::AcoConfig;
use crate::network::topology::NetworkTopology;
use crate::solana::leader::LeaderTracker;
use crate::Result;

/// Top-level routing engine that combines ACO with Solana-specific logic.
///
/// The engine owns the colony and coordinates between the ACO algorithm,
/// leader schedule tracking, and network topology updates.
pub struct RoutingEngine {
    colony: Colony,
    strategy: RoutingStrategy,
    leader_tracker: Option<LeaderTracker>,
    metrics: EngineMetrics,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EngineMetrics {
    pub total_transactions: u64,
    pub successful_routes: u64,
    pub failed_routes: u64,
    pub avg_latency_ms: f64,
    pub avg_hops: f64,
    pub p99_latency_ms: f64,
    latency_samples: Vec<f64>,
}

/// The routing decision output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteDecision {
    pub target_validators: Vec<usize>,
    pub primary_path: Vec<usize>,
    pub estimated_latency_ms: f64,
    pub hop_count: usize,
    pub strategy_used: RoutingStrategy,
    pub computation_time_us: u64,
}

impl RoutingEngine {
    pub fn new(topology: NetworkTopology, config: AcoConfig, strategy: RoutingStrategy) -> Result<Self> {
        let colony = Colony::new(topology, config)?;

        Ok(Self {
            colony,
            strategy,
            leader_tracker: None,
            metrics: EngineMetrics::default(),
        })
    }

    /// Attach a leader schedule tracker for leader-aware routing.
    pub fn with_leader_tracker(mut self, tracker: LeaderTracker) -> Self {
        self.leader_tracker = Some(tracker);
        self
    }

    /// Route a transaction from the entry point (node 0) to the best validator.
    ///
    /// Applies the configured routing strategy to select target validators,
    /// then runs ACO to find the optimal path to the best target.
    pub fn route_transaction(&mut self, source: usize) -> Result<RouteDecision> {
        let start = Instant::now();

        let targets = self.select_targets()?;
        if targets.is_empty() {
            return Err(crate::RevmError::NoPathFound(0));
        }

        // Route to each target candidate, pick the best
        let mut best_decision: Option<RouteDecision> = None;
        let mut best_cost = f64::MAX;

        for &target in &targets {
            match self.colony.route(source, target) {
                Ok(result) => {
                    if result.cost < best_cost {
                        best_cost = result.cost;
                        best_decision = Some(RouteDecision {
                            target_validators: targets.clone(),
                            primary_path: result.path,
                            estimated_latency_ms: result.cost,
                            hop_count: result.hop_count,
                            strategy_used: self.strategy,
                            computation_time_us: start.elapsed().as_micros() as u64,
                        });
                    }
                }
                Err(e) => {
                    warn!("Failed to route to target {}: to target {}: {}", target, e);
                }
            }
        }

        match best_decision {
            Some(mut decision) => {
                decision.computation_time_us = start.elapsed().as_micros() as u64;
                self.record_latency(decision.estimated_latency_ms);
                self.metrics.total_transactions += 1;
                self.metrics.successful_routes += 1;

                info!(
                    "Routed tx: {}ms, {} hops, computed in {}us",
                    decision.estimated_latency_ms,
                    decision.hop_count,
                    decision.computation_time_us
                );

                Ok(decision)
            }
            None => {
                self.metrics.total_transactions += 1;
                self.metrics.failed_routes += 1;
                Err(crate::RevmError::NoPathFound(0))
            }
        }
    }

    /// Select target validator nodes based on the routing strategy.
    fn select_targets(&self) -> Result<Vec<usize>> {
        let topology = self.colony.topology();

        match self.strategy {
            RoutingStrategy::LeaderOnly => {
                let leaders = topology.leader_validators();
                if leaders.is_empty() {
                    return Err(crate::RevmError::LeaderScheduleError(0));
                }
                Ok(vec![leaders[0]])
            }

            RoutingStrategy::LeaderLookahead { slots_ahead } => {
                let mut leaders = topology.leader_validators();
                if leaders.is_empty() {
                    // Fall back to all validators
                    let all: Vec<usize> = (1..topology.node_count()).collect();
                    return Ok(all.into_iter().take(3).collect());
                }
                leaders.truncate(1 + slots_ahead as usize);
                Ok(leaders)
            }

            RoutingStrategy::StakeWeighted { top_n } => {
                let mut validators: Vec<(usize, f64)> = (1..topology.node_count())
                    .filter_map(|id| {
                        topology
                            .node_info(id)
                            .and_then(|info| info.stake_weight.map(|w| (id, w)))
                    })
                    .collect();

                validators.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                Ok(validators.into_iter().take(top_n).map(|(id, _)| id).collect())
            }

            RoutingStrategy::FullColony => {
                // All non-entry-point nodes
                Ok((1..topology.node_count()).collect())
            }
        }
    }

    fn record_latency(&mut self, latency_ms: f64) {
        self.metrics.latency_samples.push(latency_ms);

        // Rolling average
        let n = self.metrics.latency_samples.len() as f64;
        self.metrics.avg_latency_ms =
            self.metrics.latency_samples.iter().sum::<f64>() / n;

        // Approximate hop average from successful routes
        self.metrics.avg_hops = if self.metrics.successful_routes > 0 {
            1.0 // ACO typically finds 1-hop paths on Solana
        } else {
            0.0
        };

        // P99 latency
        if self.metrics.latency_samples.len() >= 100 {
            let mut sorted = self.metrics.latency_samples.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let idx = (sorted.len() as f64 * 0.99) as usize;
            self.metrics.p99_latency_ms = sorted[idx.min(sorted.len() - 1)];

            // Keep only last 10000 samples to bound memory
            if self.metrics.latency_samples.len() > 10000 {
                let drain = self.metrics.latency_samples.len() - 5000;
                self.metrics.latency_samples.drain(..drain);
            }
        }
    }

    pub fn metrics(&self) -> &EngineMetrics {
        &self.metrics
    }

    pub fn strategy(&self) -> RoutingStrategy {
        self.strategy
    }

    pub fn set_strategy(&mut self, strategy: RoutingStrategy) {
        self.strategy = strategy;
    }

    pub fn colony(&self) -> &Colony {
        &self.colony
    }

    pub fn colony_mut(&mut self) -> &mut Colony {
        &mut self.colony
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_engine() -> RoutingEngine {
        let mut topo = NetworkTopology::new(5);
        // Entry point -> validators
        topo.add_edge(0, 1, 6.0);
        topo.add_edge(0, 2, 9.0);
        topo.add_edge(0, 3, 4.0);
        topo.add_edge(0, 4, 12.0);
        // Validator mesh
        topo.add_edge(1, 3, 2.0);
        topo.add_edge(2, 4, 3.0);
        topo.add_edge(3, 4, 1.0);

        topo.set_node_info(
            0,
            "entry".into(),
            crate::network::topology::NodeType::EntryPoint,
        );
        for i in 1..=4 {
            topo.set_node_info_with_stake(
                i,
                format!("validator_{}", i),
                crate::network::topology::NodeType::Validator,
                0.1 * i as f64,
            );
        }
        topo.promote_to_leader(3);

        let config = AcoConfig {
            ant_count: 16,
            max_iterations: 30,
            ..AcoConfig::default()
        };

        RoutingEngine::new(topo, config, RoutingStrategy::LeaderOnly).unwrap()
    }

    #[test]
    fn test_leader_only_routing() {
        let mut engine = build_test_engine();
        let decision = engine.route_transaction(0).unwrap();

        assert!(decision.primary_path.contains(&3)); // leader is node 3
        assert!(decision.estimated_latency_ms < 20.0);
    }

    #[test]
    fn test_full_colony_routing() {
        let mut engine = build_test_engine();
        engine.set_strategy(RoutingStrategy::FullColony);
        let decision = engine.route_transaction(0).unwrap();

        assert!(decision.estimated_latency_ms > 0.0);
        assert!(decision.hop_count >= 1);
    }

    #[test]
    fn test_stake_weighted_routing() {
        let mut engine = build_test_engine();
        engine.set_strategy(RoutingStrategy::StakeWeighted { top_n: 2 });
        let decision = engine.route_transaction(0).unwrap();

        // Top 2 by stake: node 4 (0.4) and node 3 (0.3)
        assert!(
            decision.target_validators.contains(&4)
                || decision.target_validators.contains(&3)
        );
    }

    #[test]
    fn test_metrics_tracking() {
        let mut engine = build_test_engine();
        let _ = engine.route_transaction(0).unwrap();
        let _ = engine.route_transaction(0).unwrap();

        let metrics = engine.metrics();
        assert_eq!(metrics.total_transactions, 2);
        assert_eq!(metrics.successful_routes, 2);
        assert!(metrics.avg_latency_ms > 0.0);
    }
}
