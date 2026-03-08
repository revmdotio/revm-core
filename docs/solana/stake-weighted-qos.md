# Stake-Weighted QoS

## What is Stake-Weighted QoS?

Solana v1.17 introduced stake-weighted Quality of Service (QoS) at the TPU layer. Validators allocate QUIC connection slots and bandwidth proportionally to the sender's stake weight. This means:

- **High-stake validators**: More connection slots, higher throughput
- **Unstaked senders**: Minimal allocation, easily rate-limited
- **Staked proxies**: Intermediate priority based on delegated stake

## How REVM Leverages Stake-Weighted QoS

### Topology Awareness

Every validator in REVM's topology carries a `stake_weight` field:

```rust
ValidatorEntry {
    pubkey: "Vote111...".into(),
    stake_weight: 0.05,  // 5% of total stake
    // ...
}
```

This weight influences ACO routing in two ways:

1. **Heuristic bonus**: Higher-stake validators get a slight heuristic advantage in ant path selection
2. **Strategy targeting**: `StakeWeighted` strategy explicitly routes to top-N validators by stake

### StakeWeighted Strategy

```rust
RoutingStrategy::StakeWeighted { top_n: 10 }
```

Selects the top 10 validators by stake as routing targets. These validators:
- Are least likely to rate-limit your transactions
- Have the most connection capacity
- Are statistically more likely to be upcoming leaders

### Stake in the ACO Formula

The latency heuristic `eta(i,j)` can incorporate stake:

```
eta(i,j) = (1 / latency) * (1 + stake_weight * latency_weight)
```

This subtly biases the colony toward high-stake validators when latency is similar, without overriding actual latency measurements.

## Practical Implications

### For Application Developers

| Scenario | Recommended Strategy |
|---|---|
| Standard transactions | `LeaderLookahead { slots_ahead: 2 }` |
| High-frequency trading | `StakeWeighted { top_n: 5 }` |
| During congestion | `StakeWeighted { top_n: 10 }` |
| Testing/development | `LeaderOnly` or `FullColony` |

### During Network Congestion

When the network is congested, stake-weighted QoS becomes critical:

1. Unstaked RPCs hit rate limits first
2. Low-stake validators have smaller connection pools
3. High-stake validators maintain throughput

REVM's adaptive routing detects congestion through increased latency and failed sends, naturally shifting toward higher-stake targets.

### Connection Pooling

For sustained transaction sending, REVM maintains QUIC connections to high-value validators:

- Connections to top-10 stake validators are kept alive
- Connection age improves priority in Solana's rate limiter
- Reconnection happens automatically on failure

## Monitoring

The `RoutingEngine` tracks stake-related metrics:

```rust
let metrics = engine.get_metrics();
// metrics.avg_target_stake — average stake of selected targets
// metrics.stake_utilization — % of routes to top-20 validators
```

This helps diagnose routing quality and stake distribution.
