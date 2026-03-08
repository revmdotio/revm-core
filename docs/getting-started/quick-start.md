# Quick Start

## Rust — Route a transaction

```rust
use revm_core::aco::{AcoConfig, Colony};
use revm_core::network::topology::{NetworkTopology, ValidatorEntry};

// 1. Build topology from cluster data
let validators = vec![
    ValidatorEntry {
        pubkey: "YourValidator111111111111111111111111111111".into(),
        stake_weight: 0.05,
        estimated_latency_ms: 6.0,
        is_leader: true,
        tpu_addr: Some("10.0.1.1:8004".into()),
    },
    ValidatorEntry {
        pubkey: "AnotherVal222222222222222222222222222222222".into(),
        stake_weight: 0.03,
        estimated_latency_ms: 10.0,
        is_leader: false,
        tpu_addr: Some("10.0.2.1:8004".into()),
    },
];

let topology = NetworkTopology::from_cluster_snapshot(
    validators,
    "rpc.mainnet-beta.solana.com"
);

// 2. Create colony with mainnet-tuned config
let config = AcoConfig::mainnet();
let mut colony = Colony::new(topology, config).unwrap();

// 3. Route from entry point (0) to leader validator (1)
let result = colony.route(0, 1).unwrap();

println!("Path: {:?}", result.path);
println!("Cost: {:.2}ms", result.cost);
println!("Hops: {}", result.hop_count);
println!("Iterations: {}", result.iterations_used);
```

## Rust — Use the routing engine

```rust
use revm_core::aco::AcoConfig;
use revm_core::router::{RoutingEngine, RoutingStrategy};
use revm_core::network::topology::NetworkTopology;

let topology = build_your_topology();
let config = AcoConfig::mainnet();

let mut engine = RoutingEngine::new(
    topology,
    config,
    RoutingStrategy::LeaderLookahead { slots_ahead: 2 },
).unwrap();

let decision = engine.route_transaction(0).unwrap();

println!("Target: {:?}", decision.target_validators);
println!("Latency: {:.1}ms", decision.estimated_latency_ms);
println!("Computed in: {}us", decision.computation_time_us);
```

## TypeScript — Send a transaction

```typescript
import { RevmClient } from 'revm-sdk';

const client = new RevmClient({
  rpcUrl: 'https://api.mainnet-beta.solana.com',
});

// Initialize — fetches validator set and builds topology
await client.initialize();

// Send with ACO-optimized routing
const result = await client.sendTransaction(
  {
    transaction: yourSignedTransaction,
    skipPreflight: true,
  },
  {
    strategy: 'leader-lookahead',
    slotsAhead: 2,
  }
);

console.log(`Signature: ${result.signature}`);
console.log(`Latency: ${result.sendLatencyMs}ms`);
console.log(`Hops: ${result.hopCount}`);
```

## TypeScript — Client-side routing only

If you just want path computation without sending:

```typescript
import { AcoRouter } from 'revm-sdk';

const router = new AcoRouter(10, {
  antCount: 32,
  maxIterations: 50,
});

// Set up edges (node 0 = entry, 1-9 = validators)
router.setEdge(0, 1, 6.0);  // 6ms to validator 1
router.setEdge(0, 2, 10.0); // 10ms to validator 2
router.setEdge(1, 3, 2.0);  // inter-validator link

const result = router.route(0, 3);
if (result) {
  console.log(`Best path: ${result.path}`);
  console.log(`Cost: ${result.cost}ms`);
}
```
