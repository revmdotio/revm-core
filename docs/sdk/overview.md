# TypeScript SDK Overview

## What It Does

The `@revm-protocol/sdk` package provides:

1. **RevmClient** — Full-featured transaction sending with ACO-optimized routing
2. **AcoRouter** — Standalone client-side ACO path computation (no network calls)

## Architecture

```
@revm-protocol/sdk
├── RevmClient        High-level: init → route → send → confirm
│   ├── topology      Built from getClusterNodes + getLeaderSchedule
│   ├── AcoRouter     Path computation engine
│   └── sender        QUIC/RPC transaction delivery
│
├── AcoRouter         Low-level: setEdge → route → read result
│   ├── pheromone     In-memory trail matrix
│   └── ants          Probabilistic path builders
│
└── types.ts          Shared type definitions
```

## Quick Comparison

| Feature | RevmClient | AcoRouter |
|---|---|---|
| Topology from RPC | Yes | No (manual) |
| Transaction sending | Yes | No |
| Leader schedule | Yes | No |
| ACO computation | Yes (internal) | Yes |
| Standalone use | Needs Solana RPC | Works anywhere |
| Dependencies | @solana/web3.js | None |

## When to Use What

**Use `RevmClient`** when you're sending transactions on Solana and want the full routing pipeline:

```typescript
import { RevmClient } from '@revm-protocol/sdk';

const client = new RevmClient({ rpcUrl: '...' });
await client.initialize();
const result = await client.sendTransaction(tx, options);
```

**Use `AcoRouter`** when you only need path computation — for analysis, visualization, or integration into your own sending logic:

```typescript
import { AcoRouter } from '@revm-protocol/sdk';

const router = new AcoRouter(nodeCount, config);
router.setEdge(0, 1, 6.0);
const result = router.route(0, 1);
```

## Installation

```bash
npm install @revm-protocol/sdk
```

Peer dependency:
```bash
npm install @solana/web3.js
```

## Requirements

- Node.js 18+
- TypeScript 5.0+ (recommended but not required)
- `@solana/web3.js ^1.87.0` (for RevmClient only)
