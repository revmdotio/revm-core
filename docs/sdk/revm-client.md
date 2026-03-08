# RevmClient

## Overview

`RevmClient` is the primary entry point for sending ACO-routed transactions on Solana. It handles topology discovery, leader tracking, route computation, and transaction delivery.

## Constructor

```typescript
import { RevmClient } from 'revm-sdk';

const client = new RevmClient({
  rpcUrl: 'https://api.mainnet-beta.solana.com',
});
```

### Options

| Option | Type | Default | Description |
|---|---|---|---|
| `rpcUrl` | `string` | required | Solana RPC endpoint URL |

## Methods

### initialize()

Fetches cluster data and builds the network topology. **Must be called before sending transactions.**

```typescript
await client.initialize();
```

This performs:
1. `getVoteAccounts` — Fetches validator set with stake weights
2. `getLeaderSchedule` — Loads current epoch leader rotation
3. `getClusterNodes` — Resolves TPU addresses
4. Builds `NetworkTopology` and initializes the ACO router

### sendTransaction(payload, options)

Routes and sends a transaction using ACO optimization.

```typescript
const result = await client.sendTransaction(
  {
    transaction: signedTransaction,  // Buffer or Uint8Array
    skipPreflight: true,
  },
  {
    strategy: 'leader-lookahead',
    slotsAhead: 2,
  }
);
```

#### Payload

| Field | Type | Description |
|---|---|---|
| `transaction` | `Buffer` | Signed, serialized transaction |
| `skipPreflight` | `boolean` | Skip RPC simulation (recommended for speed) |

#### Options

| Field | Type | Default | Description |
|---|---|---|---|
| `strategy` | `string` | `'leader-lookahead'` | Routing strategy |
| `slotsAhead` | `number` | `2` | Slots to look ahead (for leader-lookahead) |
| `topN` | `number` | `10` | Validator count (for stake-weighted) |

#### Strategy Strings

- `'leader-only'` — Current leader only
- `'leader-lookahead'` — Current + next N leaders
- `'stake-weighted'` — Top N by stake
- `'full-colony'` — All validators

#### Return Value

```typescript
interface SendResult {
  signature: string;          // Transaction signature (base58)
  sendLatencyMs: number;      // End-to-end send time
  hopCount: number;           // Network hops taken
  routeComputeUs: number;     // ACO computation time (microseconds)
  targetValidator: string;    // Pubkey of target validator
  sendMethod: 'tpu' | 'rpc'; // Delivery method used
}
```

## Usage Example

```typescript
import { RevmClient } from 'revm-sdk';
import { Keypair, Transaction, SystemProgram } from '@solana/web3.js';

const client = new RevmClient({
  rpcUrl: 'https://api.mainnet-beta.solana.com',
});

await client.initialize();

// Build your transaction
const tx = new Transaction().add(
  SystemProgram.transfer({
    fromPubkey: sender.publicKey,
    toPubkey: recipient,
    lamports: 1_000_000,
  })
);
tx.sign(sender);

// Send with ACO routing
const result = await client.sendTransaction(
  {
    transaction: tx.serialize(),
    skipPreflight: true,
  },
  {
    strategy: 'leader-lookahead',
    slotsAhead: 2,
  }
);

console.log(`Signature: ${result.signature}`);
console.log(`Latency: ${result.sendLatencyMs}ms`);
console.log(`Method: ${result.sendMethod}`);
```

## Error Handling

```typescript
try {
  const result = await client.sendTransaction(payload, options);
} catch (error) {
  if (error.message.includes('not initialized')) {
    // Forgot to call initialize()
    await client.initialize();
  }
  if (error.message.includes('no leaders found')) {
    // Leader schedule not loaded or stale
    await client.initialize(); // Re-fetch
  }
}
```

## Lifecycle

```
new RevmClient(options)
  │
  ├── initialize()        Fetch cluster data, build topology
  │
  ├── sendTransaction()   Route + send (repeatable)
  ├── sendTransaction()
  ├── sendTransaction()
  │
  └── (garbage collected)  No explicit cleanup needed
```
