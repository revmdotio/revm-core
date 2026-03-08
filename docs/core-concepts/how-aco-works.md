# How ACO Works

## Overview

Ant Colony Optimization (ACO) is a metaheuristic inspired by the foraging behavior of real ants. When ants find food, they deposit pheromone trails that other ants follow — creating a positive feedback loop that converges on the shortest path.

REVM applies this to Solana transaction routing: instead of food, ants search for the lowest-latency path to validator TPU ports.

## The Algorithm

### 1. Initialization

Every edge in the network graph starts with a small initial pheromone value (`initial_pheromone`). This ensures all paths are explored early on.

```
tau(i,j) = initial_pheromone  for all edges (i,j)
```

### 2. Ant Construction

Each ant builds a complete path from source to destination. At every node, the ant picks the next hop probabilistically:

```
P(i -> j) = [tau(i,j)^alpha * eta(i,j)^beta] / SUM_k[tau(i,k)^alpha * eta(i,k)^beta]
```

Where:
- `tau(i,j)` = pheromone intensity on edge (i,j)
- `eta(i,j)` = heuristic value = `1 / latency(i,j)`
- `alpha` = pheromone influence weight
- `beta` = heuristic influence weight

Higher pheromone and lower latency both increase selection probability.

### 3. Path Evaluation

After all ants complete their paths, each path is scored by total cost (sum of edge latencies). The global-best path is tracked across iterations.

### 4. Pheromone Evaporation

All pheromone trails decay by a fixed rate each iteration:

```
tau(i,j) = (1 - rho) * tau(i,j)
```

This prevents stagnation — old paths that are no longer optimal gradually lose their trails.

### 5. Pheromone Deposit

The best ant deposits pheromone on its path edges:

```
tau(i,j) = tau(i,j) + deposit_weight / best_cost
```

Better paths (lower cost) get proportionally more pheromone.

### 6. MMAS Bounds

REVM uses the MAX-MIN Ant System (Stutzle & Hoos, 2000) variant. Pheromone values are clamped to `[pheromone_min, pheromone_max]` after every update. This:

- Prevents total convergence on a single path (min bound)
- Prevents runaway pheromone accumulation (max bound)
- Maintains exploration throughout the algorithm's lifetime

### 7. Convergence

The colony runs for `max_iterations` or until the best path hasn't improved for several consecutive iterations.

## Adaptation to Solana

Standard ACO assumes a static graph. Solana's network is dynamic:

- **Leader rotation**: The target validator changes every ~400ms (4 slots)
- **Latency fluctuation**: Network conditions shift continuously
- **Validator churn**: Nodes join/leave the active set

REVM handles this through:

| Mechanism | How |
|---|---|
| Stale edge refresh | Re-probe edges older than `stale_threshold_ms` |
| Aggressive evaporation | Higher `evaporation_rate` on mainnet for fast adaptation |
| Leader-aware targeting | Colony routes only to current/upcoming leaders |
| Latency feedback | Actual send latency updates topology weights |

## Computational Cost

ACO runs in `O(iterations * ant_count * nodes)` time. With default mainnet settings (50 iterations, 64 ants, ~100 nodes), a full route computation completes in **<2ms** on modern hardware.
