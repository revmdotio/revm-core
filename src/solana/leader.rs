use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LeaderTracker {
    current_slot: u64,
    schedule: HashMap<u64, String>,
    pubkey_to_node: HashMap<String, usize>,
    epoch_start: u64,
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

    pub fn register_validator(&mut self, pubkey: String, node_id: usize) {
        self.pubkey_to_node.insert(pubkey, node_id);
    }

    pub fn set_current_slot(&mut self, slot: u64) {
        self.current_slot = slot;
    }

    pub fn current_slot(&self) -> u64 { self.current_slot }
}
