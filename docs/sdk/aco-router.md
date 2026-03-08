# AcoRouter

## Overview

`AcoRouter` is a standalone client-side ACO path computation engine. It's a TypeScript port of the Rust ACO algorithm — no network calls, no Solana dependency. Use it when you need path computation without transaction sending.

## Constructor

```typescript
import { AcoRouter } from 'revm-sdk';

const router = new AcoRouter(nodeCount, config);
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `nodeCount` | `number` | Total nodes in the graph |
| `config` | `AcoConfig` | Algorithm parameters (optional, has defaults) |

### Config Options

```typescript
interface AcoConfig {
  alpha?: number;          // Pheromone influence (default: 1.2)
  beta?: number;           // Latency heuristic influence (default: 3.0)
  evaporationRate?: number; // Trail decay per iteration (default: 0.25)
  antCount?: number;       // Ants per iteration (default: 32)
  maxIterations?: number;  // Convergence limit (default: 50)
}
```

## Methods

### setEdge(from, to, weight)

Add or update a directed edge in the graph.

```typescript
router.setEdge(0, 1, 6.0);   // 6ms from node 0 to node 1
router.setEdge(0, 2, 10.0);  // 10ms from node 0 to node 2
router.setEdge(1, 3, 2.0);   // 2ms inter-node link
```

Weight = latency in milliseconds. Lower is better.

### route(source, destination)

Run the ACO algorithm and return the optimal path.

```typescript
const result = router.route(0, 3);
```

Returns `null` if no path exists.

#### Result

```typescript
interface RouteResult {
  path: number[];     // Node IDs in order [0, 1, 3]
  cost: number;       // Total path cost in ms
  iterations: number; // Iterations used before convergence
}
```

### getPheromone(from, to)

Read current pheromone level on an edge.

```typescript
const tau = router.getPheromone(0, 1);
// Higher tau = more ants have used this edge recently
```

### getSnapshot()

Capture the full pheromone matrix state.

```typescript
const matrix = router.getSnapshot();
// matrix[i][j] = pheromone on edge (i, j)
```

## Usage Examples

### Basic Pathfinding

```typescript
const router = new AcoRouter(5);

// Build a simple network
router.setEdge(0, 1, 5.0);
router.setEdge(0, 2, 8.0);
router.setEdge(1, 3, 3.0);
router.setEdge(2, 3, 2.0);
router.setEdge(3, 4, 1.0);

const result = router.route(0, 4);
if (result) {
  console.log(`Path: ${result.path}`);  // [0, 1, 3, 4] or [0, 2, 3, 4]
  console.log(`Cost: ${result.cost}ms`);
}
```

### Custom Configuration

```typescript
const router = new AcoRouter(100, {
  antCount: 64,
  maxIterations: 100,
  evaporationRate: 0.3,
  alpha: 1.5,
  beta: 4.0,
});
```

### Dynamic Weight Updates

```typescript
// Initial topology
router.setEdge(0, 1, 5.0);
router.setEdge(0, 2, 8.0);

const result1 = router.route(0, 3);
// Prefers path through node 1 (lower latency)

// Network conditions change
router.setEdge(0, 1, 20.0);  // Node 1 got slower
router.setEdge(0, 2, 4.0);   // Node 2 got faster

const result2 = router.route(0, 3);
// Now prefers path through node 2
```

### Visualization Integration

```typescript
// Run routing
const result = router.route(0, target);

// Get pheromone state for visualization
const snapshot = router.getSnapshot();

// Render edges with pheromone intensity as opacity/width
for (let i = 0; i < nodeCount; i++) {
  for (let j = 0; j < nodeCount; j++) {
    if (snapshot[i][j] > 0) {
      drawEdge(i, j, { opacity: snapshot[i][j] / maxPheromone });
    }
  }
}
```

## Algorithm Details

The TypeScript implementation mirrors the Rust colony:

1. Initialize pheromone matrix with small uniform values
2. For each iteration:
   - Dispatch `antCount` ants from source
   - Each ant builds path using probability formula: `P(i->j) = tau^alpha * eta^beta / sum`
   - Track global-best path
   - Evaporate all pheromone by `evaporationRate`
   - Deposit pheromone on best path
   - Clamp to MMAS bounds
3. Return global-best path

Performance: ~1ms for 100 nodes with default config on modern browsers.
