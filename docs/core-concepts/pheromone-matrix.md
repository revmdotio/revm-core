# Pheromone Matrix

## Purpose

The pheromone matrix stores trail intensities for every edge in the network graph. It is the shared memory that allows ants to communicate indirectly — a process called **stigmergy**.

## Data Structure

```rust
pub struct PheromoneMatrix {
    tau: RwLock<Vec<Vec<f64>>>,  // NxN matrix
    config: AcoConfig,
}
```

- **NxN dense matrix** where `tau[i][j]` = pheromone on edge from node `i` to node `j`
- Protected by `parking_lot::RwLock` for thread-safe concurrent access
- Multiple ants can read simultaneously; writes are serialized

## Operations

### Initialize

All edges start at `config.initial_pheromone`:

```rust
let matrix = PheromoneMatrix::new(node_count, &config);
// All tau[i][j] = 0.05 (default initial_pheromone)
```

### Read

Ants read pheromone values during path construction:

```rust
let tau_ij = matrix.get(i, j);
```

Read operations acquire a shared lock — no contention between ants.

### Evaporate

After each iteration, all trails decay:

```rust
matrix.evaporate(evaporation_rate);
// tau[i][j] *= (1.0 - evaporation_rate) for all i,j
```

### Deposit

The best ant reinforces its path:

```rust
matrix.deposit_path(&path, deposit_amount);
// tau[i][j] += deposit_amount for each edge (i,j) in path
```

### MMAS Clamping

After every deposit and evaporation, values are clamped:

```rust
tau[i][j] = tau[i][j].clamp(config.pheromone_min, config.pheromone_max);
```

This implements MAX-MIN Ant System bounds.

## Thread Safety

The matrix uses `parking_lot::RwLock` instead of `std::sync::RwLock` for:

- No poisoning — a panicking thread won't lock out others
- Faster uncontended access
- Fair scheduling under high contention

### Access Pattern

```
Iteration start:
  [READ]  64 ants read tau values concurrently
  [WRITE] evaporate (single writer)
  [WRITE] deposit_path (single writer)
  [READ]  snapshot for diagnostics (optional)
```

## Snapshot

For debugging and visualization, you can capture the full matrix state:

```rust
let snapshot: Vec<Vec<f64>> = matrix.snapshot();
```

Returns a deep copy — the colony continues operating while you inspect the snapshot.

## Memory Usage

For N nodes, the matrix uses `N * N * 8` bytes (f64). Typical sizes:

| Validators | Matrix Size |
|---|---|
| 50 | 20 KB |
| 100 | 80 KB |
| 500 | 2 MB |
| 2000 | 32 MB |

For Solana mainnet (~2000 validators), the matrix fits comfortably in L3 cache on modern CPUs.
