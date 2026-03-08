# Leader Schedule Integration

## How Solana Leader Rotation Works

Solana assigns block production to validators in a deterministic schedule. Each validator gets 4 consecutive slots (~1.6 seconds) before rotation. The schedule is computed from stake weights at epoch boundaries and is known in advance.

For transaction routing, this means:
- You know **who** will produce the next blocks
- You can route directly to the upcoming leader's TPU port
- You can pre-compute routes before rotation happens

## LeaderTracker

```rust
pub struct LeaderTracker {
    schedule: Vec<String>,      // pubkeys indexed by slot
    current_slot: u64,
    epoch_start_slot: u64,
    slots_per_epoch: u64,
}
```

### Loading the Schedule

```rust
let tracker = LeaderTracker::new(
    schedule_data,  // from getLeaderSchedule RPC
    current_slot,
    epoch_start_slot,
    432000,         // slots per epoch (mainnet)
);
```

### Current Leader

```rust
let leader_pubkey = tracker.current_leader();
// Returns the pubkey of the validator producing the current slot
```

### Lookahead

```rust
let upcoming = tracker.leaders_ahead(3);
// Returns pubkeys for current slot + next 3 slot leaders
// Example: ["Val_A", "Val_A", "Val_B", "Val_B"]
// (Val_A has 2 remaining slots in their 4-slot window)
```

### Epoch Boundary Detection

```rust
if tracker.near_epoch_boundary(100) {
    // Within 100 slots of epoch end — schedule may change
    // Refresh schedule from RPC
}
```

## Integration with Routing

The `RoutingEngine` uses `LeaderTracker` to map leader pubkeys to topology node IDs:

```
1. LeaderTracker.leaders_ahead(slots_ahead)
2. Map pubkeys -> topology node IDs
3. Set those nodes as LeaderValidator in topology
4. ACO colony routes to LeaderValidator nodes
```

### LeaderLookahead Strategy Flow

```
Slot 1000: Leader = Val_A (nodes [3])
Slot 1001: Leader = Val_A (nodes [3])
Slot 1002: Leader = Val_B (nodes [7])
Slot 1003: Leader = Val_B (nodes [7])

With slots_ahead=2:
  Targets = {3, 7}  (unique leaders in window)
  Colony runs ACO to node 3 AND node 7
  Returns best path among both targets
```

## Refresh Strategy

The leader schedule is fetched once per epoch and cached. Near epoch boundaries, REVM proactively refreshes:

| Condition | Action |
|---|---|
| New epoch detected | Full schedule refresh |
| Within 100 slots of boundary | Pre-fetch next epoch schedule |
| Schedule miss (unknown slot) | Emergency refresh |

## RPC Calls Used

- `getLeaderSchedule` — Full epoch schedule
- `getSlot` — Current slot number
- `getEpochInfo` — Epoch boundaries and timing
