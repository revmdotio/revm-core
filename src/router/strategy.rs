use serde::{Deserialize, Serialize};

/// Routing strategy determines how the engine selects validator targets.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RoutingStrategy {
    /// Route to the current slot leader only.
    /// Lowest latency, but fails if leader is unreachable.
    LeaderOnly,

    /// Route to current leader + next 2 leaders in schedule.
    /// Covers slot boundaries where leader rotates mid-flight.
    LeaderLookahead { slots_ahead: u8 },

    /// Route to top-N validators by stake weight.
    /// Stake-weighted QoS means higher-staked validators process faster.
    StakeWeighted { top_n: usize },

    /// Full ACO optimization across all reachable validators.
    /// Highest quality routing but more compute per cycle.
    FullColony,
}

impl Default for RoutingStrategy {
    fn default() -> Self {
        Self::LeaderLookahead { slots_ahead: 2 }
    }
}

impl RoutingStrategy {
    /// Whether this strategy requires leader schedule data.
    pub fn needs_leader_schedule(&self) -> bool {
        matches!(
            self,
            Self::LeaderOnly | Self::LeaderLookahead { .. }
        )
    }

    /// Whether this strategy requires stake weight data.
    pub fn needs_stake_weights(&self) -> bool {
        matches!(self, Self::StakeWeighted { .. })
    }

    /// Maximum number of target validators this strategy will consider.
    pub fn max_targets(&self) -> usize {
        match self {
            Self::LeaderOnly => 1,
            Self::LeaderLookahead { slots_ahead } => 1 + *slots_ahead as usize,
            Self::StakeWeighted { top_n } => *top_n,
            Self::FullColony => usize::MAX,
        }
    }
}
