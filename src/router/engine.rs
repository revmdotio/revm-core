use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::time::Instant;

use super::strategy::RoutingStrategy;
use crate::aco::colony::Colony;
use crate::aco::config::AcoConfig;
use crate::network::topology::NetworkTopology;
use crate::solana::leader::LeaderTracker;
use crate::Result;

pub struct RoutingEngine {
    colony: Colony,
    strategy: RoutingStrategy,
    leader_tracker: Option<LeaderTracker>,
}

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
        Ok(Self { colony, strategy, leader_tracker: None })
    }

    pub fn with_leader_tracker(mut self, tracker: LeaderTracker) -> Self {
        self.leader_tracker = Some(tracker);
        self
    }

    pub fn strategy(&self) -> RoutingStrategy { self.strategy }
    pub fn set_strategy(&mut self, strategy: RoutingStrategy) { self.strategy = strategy; }
    pub fn colony(&self) -> &Colony { &self.colony }
    pub fn colony_mut(&mut self) -> &mut Colony { &mut self.colony }
}
