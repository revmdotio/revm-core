# Routing Strategies

## Overview

A routing strategy determines **which validators** the ACO colony targets. The colony finds the optimal *path* to those targets — the strategy picks the targets themselves.

```rust
pub enum RoutingStrategy {
    LeaderOnly,
    LeaderLookahead { slots_ahead: u64 },
    StakeWeighted { top_n: usize },
    FullColony,
}
```

## Strategies

### LeaderOnly

Routes exclusively to the current slot leader.

```rust
RoutingStrategy::LeaderOnly
```

**Target selection**: Single validator currently producing blocks.

**Best for**: Minimum latency when you know the leader won't rotate before your transaction lands.

**Risk**: If the leader rotates during transit, the transaction must be retried.

### LeaderLookahead

Routes to the current leader plus leaders for the next N slots.

```rust
RoutingStrategy::LeaderLookahead { slots_ahead: 2 }
```

**Target selection**: Current leader + next `slots_ahead` leaders from the schedule.

**Best for**: Production transaction sending. Covers leader rotation during transit.

**Trade-off**: Slightly more compute (multiple ACO targets) but much higher landing rate.

**Recommended**: `slots_ahead: 2` covers ~800ms of leader rotation — sufficient for most transactions.

### StakeWeighted

Routes to the top N validators by stake weight, regardless of leader status.

```rust
RoutingStrategy::StakeWeighted { top_n: 10 }
```

**Target selection**: Top `top_n` validators sorted by `stake_weight` descending.

**Best for**: When you want stake-weighted QoS without depending on leader schedule accuracy.

**Note**: Higher-stake validators have priority in Solana's QUIC rate limiter, so sending to them can improve landing rates even when they're not the current leader.

### FullColony

No target filtering — the colony optimizes routes to all validators.

```rust
RoutingStrategy::FullColony
```

**Target selection**: Every validator in the topology.

**Best for**: Benchmarking, analysis, or when you want to pre-compute routes to all validators.

**Warning**: Most expensive strategy. With 2000 validators, this runs 2000 ACO optimizations.

## Strategy Comparison

| Strategy | Targets | Compute | Landing Rate | Latency |
|---|---|---|---|---|
| LeaderOnly | 1 | Lowest | Medium | Lowest |
| LeaderLookahead(2) | 2-3 | Low | High | Low |
| StakeWeighted(10) | 10 | Medium | High | Medium |
| FullColony | All | Highest | Highest | Varies |

## Using Strategies

### With RoutingEngine

```rust
let mut engine = RoutingEngine::new(
    topology,
    config,
    RoutingStrategy::LeaderLookahead { slots_ahead: 2 },
).unwrap();

let decision = engine.route_transaction(0).unwrap();
```

### Changing Strategy at Runtime

The engine's strategy can be swapped between calls:

```rust
// Switch to stake-weighted during high congestion
engine.set_strategy(RoutingStrategy::StakeWeighted { top_n: 10 });
```

### TypeScript SDK

```typescript
const result = await client.sendTransaction(
  { transaction: tx, skipPreflight: true },
  { strategy: 'leader-lookahead', slotsAhead: 2 }
);
```

Available strategy strings: `'leader-only'`, `'leader-lookahead'`, `'stake-weighted'`, `'full-colony'`.
