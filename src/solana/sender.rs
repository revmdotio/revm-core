use log::{info, warn};
use std::time::Instant;

use super::types::{ClusterConfig, SendResult, TransactionPayload};
use crate::router::engine::{RouteDecision, RoutingEngine};
use crate::Result;

pub struct TransactionSender {
    cluster: ClusterConfig,
    engine: RoutingEngine,
    use_tpu: bool,
}

impl TransactionSender {
    pub fn new(cluster: ClusterConfig, engine: RoutingEngine) -> Self {
        Self { cluster, engine, use_tpu: true }
    }

    pub fn disable_tpu(&mut self) { self.use_tpu = false; }
    pub fn engine(&self) -> &RoutingEngine { &self.engine }
    pub fn engine_mut(&mut self) -> &mut RoutingEngine { &mut self.engine }
}
