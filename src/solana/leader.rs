use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tracks the Solana leader schedule and maps slot leaders to topology nodes.
///
/// The leader schedule rotates every 4 slots (~1.6s). Accurate leader tracking
/// is critical for the LeaderOnly and LeaderLookahead strategies — sending a
/// transaction to a non-leader validator adds at least one extra hop.
#[derive(Debug, Clone)]
pub struct LeaderTracker {
    /// Current slot (updated via RPC polling or websocket)
    current_slot: u64,
    /// Leader schedule: slot -> validator pubkey
    schedule: HashMap<u64, String>,
    /// Validator pubkey -> topology node ID
    pubkey_to_node: HashMap<String, usize>,
    /// Epoch start slot
    epoch_start: u64,
    /// Slots per epoch (typically 432000 on mainnet)
    slots_per_epoch: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderScheduleEntry {
    pub slot: u64,
    pub leader_pubkey: String,
}

impl LeaderTracker {
    pub fn new(slots_per_epoch: u64) -> Self {
        Self {
            current_slot: 0,
            schedule: HashMap::new(),
            pubkey_to_node: HashMap::new(),
            epoch_start: 0,
            slots_per_epoch,
        }
    }

    /// Register a validator pubkey -> topology node ID mapping.
    pub fn register_validator(&mut self, pubkey: String, node_id: usize) {
        self.pubkey_to_node.insert(pubkey, node_id);
    }

    /// Load the leader schedule for the current epoch.
    pub fn load_schedule(&mut self, entries: Vec<LeaderScheduleEntry>, epoch_start: u64) {
        self.epoch_start = epoch_start;
        self.schedule.clear();
        for entry in entries {
            self.schedule.insert(entry.slot, entry.leader_pubkey);
        }
        debug!(
            "Loaded leader schedule: {} entries starting at slot {}",
            self.schedule.len(),
            epoch_start
        );
    }

    /// Update the current slot.
    pub fn set_current_slot(&mut self, slot: u64) {
        self.current_slot = slot;
    }

    /// Get the leader node ID for the current slot.
    pub fn current_leader(&self) -> Option<usize> {
        self.leader_at_slot(self.current_slot)
    }

    /// Get the leader node ID for a specific slot.
    pub fn leader_at_slot(&self, slot: u64) -> Option<usize> {
        self.schedule
            .get(&slot)
            .and_then(|pubkey| self.pubkey_to_node.get(pubkey))
            .copied()
    }

    /// Get leader node IDs for the next N slots (lookahead).
    pub fn leaders_ahead(&self, count: u8) -> Vec<usize> {
        let mut leaders = Vec::with_capacity(count as usize + 1);
        let mut seen = std::collections::HashSet::new();

        for offset in 0..=(count as u64) {
            let slot = self.current_slot + offset;
            if let Some(node_id) = self.leader_at_slot(slot) {
                if seen.insert(node_id) {
                    leaders.push(node_id);
                }
            }
        }

        if leaders.is_empty() {
            warn!(
                "No leaders found in schedule for slots {}..{}",
                self.current_slot,
                self.current_slot + count as u64
            );
        }

        leaders
    }

    /// Check if the schedule needs refresh (approaching epoch boundary).
    pub fn needs_refresh(&self) -> bool {
        let epoch_end = self.epoch_start + self.slots_per_epoch;
        let remaining = epoch_end.saturating_sub(self.current_slot);
        // Refresh when less than 1000 slots remain in the epoch
        remaining < 1000 || self.schedule.is_empty()
    }

    pub fn current_slot(&self) -> u64 {
        self.current_slot
    }

    pub fn current_epoch(&self) -> u64 {
        if self.slots_per_epoch == 0 {
            return 0;
        }
        self.current_slot / self.slots_per_epoch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_tracker() -> LeaderTracker {
        let mut tracker = LeaderTracker::new(432000);
        tracker.register_validator("LeaderA1111111111111111111111111111111111111".into(), 1);
        tracker.register_validator("LeaderB2222222222222222222222222222222222222".into(), 2);
        tracker.register_validator("LeaderC3333333333333333333333333333333333333".into(), 3);

        let entries = vec![
            LeaderScheduleEntry { slot: 100, leader_pubkey: "LeaderA1111111111111111111111111111111111111".into() },
            LeaderScheduleEntry { slot: 101, leader_pubkey: "LeaderA1111111111111111111111111111111111111".into() },
            LeaderScheduleEntry { slot: 102, leader_pubkey: "LeaderB2222222222222222222222222222222222222".into() },
            LeaderScheduleEntry { slot: 103, leader_pubkey: "LeaderC3333333333333333333333333333333333333".into() },
            LeaderScheduleEntry { slot: 104, leader_pubkey: "LeaderB2222222222222222222222222222222222222".into() },
        ];
        tracker.load_schedule(entries, 0);
        tracker.set_current_slot(100);
        tracker
    }

    #[test]
    fn test_current_leader() {
        let tracker = build_tracker();
        assert_eq!(tracker.current_leader(), Some(1));
    }

    #[test]
    fn test_leader_at_slot() {
        let tracker = build_tracker();
        assert_eq!(tracker.leader_at_slot(102), Some(2));
        assert_eq!(tracker.leader_at_slot(103), Some(3));
        assert_eq!(tracker.leader_at_slot(999), None);
    }

    #[test]
    fn test_leaders_ahead() {
        let tracker = build_tracker();
        let ahead = tracker.leaders_ahead(3);
        // Slots 100-103: A(1), A(1), B(2), C(3) -> deduplicated: [1, 2, 3]
        assert_eq!(ahead, vec![1, 2, 3]);
    }

    #[test]
    fn test_needs_refresh() {
        let mut tracker = LeaderTracker::new(432000);
        assert!(tracker.needs_refresh()); // empty schedule

        tracker.load_schedule(vec![], 0);
        tracker.set_current_slot(431500);
        assert!(tracker.needs_refresh()); // near epoch end
    }
}
