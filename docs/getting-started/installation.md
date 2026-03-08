# Installation

## Rust

> [revm-core on crates.io](https://crates.io/crates/revm-core)

Add `revm-core` to your `Cargo.toml`:

```toml
[dependencies]
revm-core = "0.1.0"
```

### Requirements

- Rust 1.73 or later
- Solana CLI tools (optional, for local testing)

### Build from source

```bash
git clone https://github.com/revmdotio/revm-core.git
cd revm-core
cargo build --release
```

### Run tests

```bash
cargo test --all
```

## TypeScript SDK

> [revm-sdk on npm](https://www.npmjs.com/package/revm-sdk)

```bash
npm install revm-sdk
```

### Requirements

- Node.js 18+
- TypeScript 5.0+ (recommended)

### Peer dependencies

The SDK depends on `@solana/web3.js ^1.87.0`. If you don't already have it:

```bash
npm install @solana/web3.js
```
