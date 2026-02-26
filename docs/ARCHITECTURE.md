# Architecture

## Core Design Decisions

### Why ACO for Transaction Routing

Traditional Solana transaction delivery sends transactions through a single RPC endpoint and hopes for the best. This ignores network topology, validator proximity, and leader schedule timing. ACO provides adaptive, self-optimizing routing that learns from every transaction sent.

### Pheromone as Network Memory

The pheromone matrix acts as a distributed memory of past routing success. Edges that consistently deliver low-latency results accumulate pheromone, naturally guiding future transactions along proven paths. The evaporation mechanism ensures the system adapts when network conditions change (leader rotation, validator downtime, congestion).

### MMAS Bounds

We use Max-Min Ant System bounds to prevent two failure modes:
- **Stagnation**: Without a pheromone maximum, a single dominant path accumulates all pheromone and the colony stops exploring alternatives.
- **Extinction**: Without a pheromone minimum, underused paths lose all pheromone and become permanently invisible to ants, even if they become optimal later.

### Leader-Aware Routing

Solana's leader schedule rotates every 4 slots (~1.6 seconds). Sending a transaction to a non-leader validator adds at least one forwarding hop and increases confirmation latency. The `LeaderLookahead` strategy accounts for the possibility that the leader rotates while the transaction is in flight.

### Stake-Weighted QoS Integration

Solana implements stake-weighted quality of service at the TPU level. Transactions arriving at high-stake validators through their QUIC endpoints receive priority processing. The `StakeWeighted` strategy leverages this by preferring high-stake validators as routing targets.

## Data Flow

```
Transaction Submit
       |
       v
  RoutingEngine.route_transaction()
       |
       +---> select_targets() based on strategy
       |         |
       |         +---> LeaderTracker (leader schedule)
       |         +---> Topology (stake weights)
       |
       +---> Colony.route(source, target) for each candidate
       |         |
       |         +---> Ant.find_path() x ant_count
       |         +---> PheromoneMatrix.evaporate()
       |         +---> PheromoneMatrix.deposit_path() (best ant)
       |
       +---> Pick lowest-cost route
       |
       v
  TransactionSender.send()
       |
       +---> TPU/QUIC (preferred)
       +---> RPC fallback
       |
       v
  Feedback: measured latency -> Topology.update_latency()
```

## Thread Safety

- `PheromoneMatrix` uses `parking_lot::RwLock` for concurrent read access during ant pathfinding with exclusive write access during evaporation and deposit phases.
- `NetworkTopology` is cloned per routing cycle to avoid contention between the probe thread and routing thread.
- The `Colony` itself is not `Send+Sync` by design. Each thread that needs routing should own its own colony instance or use the `RoutingEngine` which manages this internally.
