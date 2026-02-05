# revm-core

<p align="center">
  <img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="MIT" />
  <img src="https://img.shields.io/badge/rust-1.73+-orange?style=flat-square&logo=rust" alt="Rust" />
  <img src="https://img.shields.io/badge/solana-1.17-9945FF?style=flat-square&logo=solana" alt="Solana" />
</p>

Ant Colony Optimization routing engine for Solana transaction delivery.
Sub-10ms path selection. Single-hop delivery. Zero MEV exposure.

## Overview

REVM Core applies Ant Colony Optimization to Solana transaction routing. Instead of sending transactions blindly through a single RPC, it discovers optimal paths to validator TPU ports using pheromone-based pathfinding.

Based on Dorigo's Ant System (1996) and AntNet (Di Caro & Dorigo, 1998).

## Install

```toml
[dependencies]
revm-core = "0.1.0"
```

## License

MIT
