# revm-sdk

TypeScript SDK for the REVM Ant Colony Optimization routing protocol on Solana.

Routes transactions through optimal validator paths using bio-inspired ACO algorithms, minimizing latency and MEV exposure.

## Installation

```bash
npm install revm-sdk @solana/web3.js
```

## Quick Start

```typescript
import { RevmClient } from 'revm-sdk';
import { Connection, Keypair, Transaction, SystemProgram, LAMPORTS_PER_SOL } from '@solana/web3.js';

const client = new RevmClient({
  rpcUrl: 'https://api.mainnet-beta.solana.com',
});

await client.initialize();

// Build your transaction
const tx = new Transaction().add(
  SystemProgram.transfer({
    fromPubkey: sender.publicKey,
    toPubkey: receiver,
    lamports: 0.01 * LAMPORTS_PER_SOL,
  })
);

// Send with ACO-optimized routing
const result = await client.sendTransaction(
  { transaction: tx },
  { strategy: 'leader-lookahead' }
);

console.log(`Signature: ${result.signature}`);
console.log(`Latency: ${result.sendLatencyMs.toFixed(1)}ms`);
console.log(`Hops: ${result.hopCount}`);
console.log(`Target: ${result.targetValidator}`);
```

## Routing Strategies

| Strategy | Description |
|----------|-------------|
| `leader-lookahead` | Routes to the best of current + upcoming slot leaders (default) |
| `leader-only` | Routes directly to the current slot leader |
| `stake-weighted` | Routes to the highest-stake validator with the best ACO path |
| `full-colony` | Runs full colony optimization across all validators |

```typescript
// Leader lookahead with 4 slots ahead
const result = await client.sendTransaction(
  { transaction: tx },
  { strategy: 'leader-lookahead', slotsAhead: 4 }
);

// Stake-weighted routing, top 10 validators
const result = await client.sendTransaction(
  { transaction: tx },
  { strategy: 'stake-weighted', topN: 10 }
);
```

## ACO Configuration

```typescript
const client = new RevmClient({
  rpcUrl: 'https://api.mainnet-beta.solana.com',
  acoConfig: {
    alpha: 1.2,           // Pheromone influence
    beta: 3.0,            // Latency heuristic influence
    evaporationRate: 0.25, // Pheromone decay per iteration
    antCount: 32,          // Ants per iteration
    maxIterations: 50,     // Max ACO iterations
  },
});
```

## API Reference

### RevmClient

#### `new RevmClient(config: RevmClientConfig)`

Creates a new REVM client instance.

#### `client.initialize(): Promise<void>`

Fetches validator data and builds the ACO routing topology. Must be called before sending transactions.

#### `client.sendTransaction(payload, options?): Promise<SendResult>`

Sends a transaction using ACO-optimized routing.

**Payload:**
- `transaction` — `Transaction | VersionedTransaction`
- `skipPreflight?` — Skip preflight simulation (default: `false`)
- `maxRetries?` — Max send retries (default: `3`)

**Options:**
- `strategy?` — Routing strategy (default: `'leader-lookahead'`)
- `slotsAhead?` — Slots to look ahead for leaders (default: `2`)
- `topN?` — Top N validators for stake-weighted (default: `5`)

**Returns `SendResult`:**
- `signature` — Transaction signature
- `targetValidator` — Selected validator pubkey
- `sendLatencyMs` — Send latency in milliseconds
- `hopCount` — Number of hops in the route
- `slot` — Current slot
- `confirmed` — Confirmation status

#### `client.confirmTransaction(signature, timeout?): Promise<boolean>`

Waits for transaction confirmation.

#### `client.getMetrics(): ColonyMetrics`

Returns routing performance metrics.

#### `client.getValidators(): ValidatorNode[]`

Returns the current validator topology.

### AcoRouter

Standalone ACO router for custom routing logic.

```typescript
import { AcoRouter } from 'revm-sdk';

const router = new AcoRouter(10, {
  alpha: 1.2,
  beta: 3.0,
  evaporationRate: 0.25,
  antCount: 32,
  maxIterations: 50,
});

router.setEdge(0, 1, 5.0);  // node 0 -> node 1, 5ms latency
router.setEdge(0, 2, 8.0);
router.setEdge(1, 3, 3.0);
router.setEdge(2, 3, 2.0);

const result = router.route(0, 3);
// { path: [0, 2, 3], cost: 10.0, hopCount: 2, iterationsUsed: 15 }
```

## Companion Crate

The core ACO engine is written in Rust for maximum performance:

```bash
cargo add revm-core
```

See [revm-core on crates.io](https://crates.io/crates/revm-core) for the Rust implementation.

## Links

- [Website](https://revm.io)
- [Whitepaper](https://revm.io/whitepaper)
- [GitHub](https://github.com/revmdotio/revm-core)
- [crates.io](https://crates.io/crates/revm-core)

## License

MIT
