<div class="docs-hero">
  <img src="https://raw.githubusercontent.com/revmdotio/revm-core/main/docs/docs.png" alt="REVM Protocol">
</div>

# REVM Core

Ant Colony Optimization routing engine for Solana transaction delivery.

## What is REVM?

REVM applies Ant Colony Optimization (ACO) to Solana transaction routing. Instead of sending transactions blindly through a single RPC endpoint, REVM discovers optimal paths to validator TPU ports using pheromone-based pathfinding across the network topology.

The result: **sub-10ms path selection, single-hop delivery, zero MEV exposure.**

## Why ACO?

Traditional Solana transaction delivery sends transactions through a single RPC and hopes for the best. This ignores:

- Network topology and validator proximity
- Leader schedule rotation timing
- Latency variations across different paths
- MEV observation surfaces at relay nodes

ACO solves all of these by maintaining a living pheromone map of the network. Good paths get reinforced. Bad paths evaporate. The colony continuously adapts.

## Key Numbers

| Metric | Value |
|---|---|
| Average routing computation | < 2ms |
| Average send latency | ~9ms |
| Typical hop count | 1 (direct to leader TPU) |
| MEV exposure | Zero |
| Convergence time | < 2 seconds |

## Language Split

- **Rust** (70%+) — Core ACO engine, routing, Solana integration
- **TypeScript** (25%) — Client SDK with @solana/web3.js integration

## Academic Foundation

Based on 30 years of peer-reviewed research:

- Dorigo et al., 1996 — Ant System (IEEE, 18K+ citations)
- Di Caro & Dorigo, 1998 — AntNet (JAIR, 2.8K+ citations)
- Stutzle & Hoos, 2000 — MAX-MIN Ant System
