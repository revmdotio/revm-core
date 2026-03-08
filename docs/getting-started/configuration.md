# Configuration

## AcoConfig

The `AcoConfig` struct controls all ACO algorithm parameters.

```rust
AcoConfig {
    alpha: 1.2,              // pheromone trail influence
    beta: 3.0,               // latency heuristic influence
    evaporation_rate: 0.25,  // trail decay per iteration
    initial_pheromone: 0.05, // starting pheromone on all edges
    pheromone_min: 0.001,    // MMAS lower bound
    pheromone_max: 8.0,      // MMAS upper bound
    ant_count: 64,           // agents dispatched per cycle
    max_iterations: 50,      // convergence limit
    deposit_weight: 1.5,     // best-ant pheromone deposit multiplier
    latency_weight: 1.2,     // heuristic scaling factor
    stale_threshold_ms: 1200 // re-probe edges older than this
}
```

## Presets

### Mainnet

`AcoConfig::mainnet()` — Aggressive evaporation to track leader rotation within ~2 slots. Higher ant count for faster convergence. Tight stale threshold for fresh latency data.

Best for: production transaction sending on mainnet-beta.

### Devnet

`AcoConfig::devnet()` — Relaxed timing, more exploration. Higher pheromone bounds to allow more diverse paths. Longer stale threshold since devnet conditions are more stable.

Best for: testing, development, integration tests.

### Custom

Create your own config for specific use cases:

```rust
let config = AcoConfig {
    ant_count: 128,          // more ants = better paths, more compute
    max_iterations: 20,      // fewer iterations = faster but less optimal
    evaporation_rate: 0.30,  // higher = faster adaptation, less memory
    ..AcoConfig::mainnet()   // inherit other mainnet defaults
};
```

## Parameter Guide

| Parameter | Low Value | High Value | Trade-off |
|---|---|---|---|
| `alpha` | More exploration | More exploitation | Path diversity vs. convergence speed |
| `beta` | Ignore latency | Latency-dominated | Exploration vs. greedy selection |
| `evaporation_rate` | Long memory | Short memory | Stability vs. adaptability |
| `ant_count` | Faster compute | Better paths | Speed vs. quality |
| `max_iterations` | Quick result | Optimal result | Latency vs. accuracy |

## Validation

All configs are validated before use:

```rust
let config = AcoConfig { evaporation_rate: 1.5, ..Default::default() };
assert!(config.validate().is_err()); // must be in (0.0, 1.0)
```

Rules:
- `alpha` >= 0
- `beta` >= 0
- `evaporation_rate` in (0.0, 1.0) exclusive
- `pheromone_min` < `pheromone_max`
- `ant_count` >= 1
