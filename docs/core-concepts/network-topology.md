# Network Topology

## Overview

The `NetworkTopology` is a weighted directed graph representing the Solana network from the perspective of a transaction sender. Nodes are validators, relays, or entry points. Edge weights are measured latencies in milliseconds.

## Node Types

```rust
pub enum NodeType {
    EntryPoint,       // Your application's origin
    Relay,            // Intermediate hop (RPC node, proxy)
    Validator,        // Stake-bearing validator
    LeaderValidator,  // Current or upcoming slot leader
}
```

### EntryPoint

Node 0 is always the entry point — your application or RPC endpoint. All routing starts here.

### Validator vs LeaderValidator

Leader status is ephemeral. The topology tracks which validators are currently leaders based on the Solana leader schedule. When the schedule rotates, `LeaderValidator` nodes become `Validator` and new leaders are promoted.

## Building a Topology

### From Cluster Snapshot

The most common method — build from validator data:

```rust
let validators = vec![
    ValidatorEntry {
        pubkey: "Vote111...".into(),
        stake_weight: 0.05,
        estimated_latency_ms: 6.0,
        is_leader: true,
        tpu_addr: Some("10.0.1.1:8004".into()),
    },
    // ...
];

let topology = NetworkTopology::from_cluster_snapshot(
    validators,
    "rpc.mainnet-beta.solana.com"
);
```

This creates:
- Node 0: EntryPoint (your position)
- Node 1..N: Validators with edges from entry point weighted by `estimated_latency_ms`

### Manual Construction

For testing or custom topologies:

```rust
let mut topo = NetworkTopology::new(5);
topo.set_node_type(0, NodeType::EntryPoint);
topo.set_node_type(1, NodeType::Validator);
topo.set_edge(0, 1, 6.0);  // 6ms from entry to validator 1
topo.set_edge(1, 2, 2.0);  // 2ms inter-validator link
```

## Edge Weights

Edge weights represent **latency in milliseconds**. Lower is better. The ACO heuristic uses `eta(i,j) = 1.0 / weight(i,j)`, so lower-latency edges are naturally preferred.

### Updating Weights

Latency measurements update edges in-place:

```rust
topology.update_edge(0, 3, 8.5);  // measured 8.5ms to validator 3
```

The feedback loop works like this:

```
Send transaction -> Measure actual latency -> Update edge weight -> Next ACO run uses new weight
```

## Leader Management

```rust
// Promote a validator to leader
topology.set_leader(node_id);

// Demote back to regular validator
topology.clear_leader(node_id);

// Get all current leaders
let leaders: Vec<usize> = topology.get_leaders();
```

Leader information feeds into routing strategies. `LeaderOnly` and `LeaderLookahead` strategies only consider leader nodes as targets.

## Graph Properties

- **Directed**: Edge (A->B) can have different weight than (B->A)
- **Dense connectivity**: Entry point connects to all validators
- **Inter-validator links**: Optional edges between validators for multi-hop paths
- **Dynamic weights**: Edge weights change as latency measurements arrive
- **No self-loops**: `topology.set_edge(i, i, _)` is a no-op
