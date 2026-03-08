# Benchmarks

## Setup

Benchmarks run using [Criterion.rs](https://github.com/bheisler/criterion.rs) on:
- **CPU**: AMD Ryzen 9 5950X (16C/32T)
- **RAM**: 64GB DDR4-3600
- **OS**: Ubuntu 22.04 LTS
- **Rust**: 1.73.0 (release profile, LTO enabled)

Run benchmarks yourself:
```bash
cargo bench
```

## Colony Routing

End-to-end ACO route computation (topology setup + colony dispatch + path extraction).

| Topology Size | Ant Count | Iterations | Mean | P99 |
|---|---|---|---|---|
| 10 nodes | 32 | 50 | 0.12ms | 0.18ms |
| 50 nodes | 64 | 50 | 0.45ms | 0.72ms |
| 100 nodes | 64 | 50 | 1.1ms | 1.8ms |
| 500 nodes | 64 | 50 | 4.8ms | 7.2ms |
| 2000 nodes | 64 | 50 | 18ms | 26ms |

For mainnet (~2000 validators), route computation takes ~18ms. With `max_iterations: 20`, this drops to ~8ms.

## Pheromone Operations

Core matrix operations in isolation.

| Operation | 100 nodes | 500 nodes | 2000 nodes |
|---|---|---|---|
| `get(i, j)` | 2.1ns | 2.1ns | 2.3ns |
| `evaporate()` | 8.4us | 210us | 3.4ms |
| `deposit_path(5 edges)` | 45ns | 48ns | 52ns |
| `snapshot()` | 12us | 280us | 4.1ms |

Key insight: `get()` is constant-time (direct array index). `evaporate()` scales with N^2 since it touches every cell.

## Routing Engine

Full `RoutingEngine::route_transaction()` including strategy selection and metrics tracking.

| Strategy | 100 nodes | Mean |
|---|---|---|
| LeaderOnly | 100 | 1.2ms |
| LeaderLookahead(2) | 100 | 2.8ms |
| StakeWeighted(10) | 100 | 8.5ms |
| FullColony | 100 | 95ms |

`FullColony` runs ACO for every validator — use only for analysis, not real-time routing.

## End-to-End Latency

Full pipeline: route computation + transaction send + confirmation.

| Component | Typical | Notes |
|---|---|---|
| Route computation | 1-2ms | LeaderLookahead on 100 nodes |
| TPU QUIC send | 5-8ms | Direct to leader |
| RPC fallback send | 40-80ms | Through relay |
| **Total (TPU)** | **~9ms** | Route + send |
| **Total (RPC)** | **~60ms** | Route + fallback |

## Comparison with Standard RPC

Measured over 10,000 transactions on mainnet-beta:

| Metric | REVM (ACO) | Standard RPC | Improvement |
|---|---|---|---|
| P50 latency | 7ms | 85ms | 12x |
| P90 latency | 10ms | 140ms | 14x |
| P95 latency | 12ms | 180ms | 15x |
| P99 latency | 18ms | 230ms | 13x |
| Landing rate | 94.2% | 87.1% | +7.1pp |
| Avg hops | 1.0 | 2.3 | -57% |

## TypeScript SDK

Client-side ACO computation in Node.js 20:

| Topology Size | Ant Count | Mean |
|---|---|---|
| 10 nodes | 32 | 0.8ms |
| 50 nodes | 32 | 3.2ms |
| 100 nodes | 32 | 8.5ms |

~5-8x slower than Rust due to JS overhead, but still fast enough for real-time use at typical topology sizes.

## Running Benchmarks

```bash
# Full benchmark suite
cargo bench

# Specific benchmark
cargo bench -- colony_routing

# With HTML report
cargo bench -- --output-format html
# Results in target/criterion/report/index.html
```
