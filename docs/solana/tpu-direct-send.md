# TPU Direct Send

## Why Direct TPU?

Standard RPC transaction flow:

```
App -> RPC Node -> Relay -> Leader Validator
       (50ms)     (30ms)     (process)
```

REVM's direct TPU flow:

```
App -> Leader TPU Port
       (5-10ms)
```

By sending directly to the leader's TPU (Transaction Processing Unit) port, REVM eliminates relay hops and reduces latency by 5-20x.

## QUIC Protocol

Since Solana v1.17, TPU connections use **QUIC** (not UDP). QUIC provides:

- **Stake-weighted rate limiting**: Higher-stake senders get priority
- **Connection authentication**: Prevents IP spoofing
- **Congestion control**: Built-in flow management
- **Multiplexing**: Multiple streams on a single connection

### Connection Flow

```
1. Resolve leader's TPU QUIC address (port 8009)
2. Establish QUIC connection with TLS
3. Open unidirectional stream
4. Send serialized transaction bytes
5. Stream closes on completion
```

## TransactionSender

```rust
pub struct TransactionSender {
    client: reqwest::Client,
    cluster_config: ClusterConfig,
}
```

### Send Methods

**TPU Direct** (primary):
```rust
let result = sender.send_to_tpu(
    &transaction_bytes,
    "10.0.1.1:8009",  // leader TPU QUIC address
).await?;
```

**RPC Fallback** (backup):
```rust
let result = sender.send_via_rpc(
    &transaction_payload,
).await?;
```

### Fallback Logic

REVM attempts TPU direct send first. If it fails (connection timeout, rate limited, etc.), it falls back to RPC:

```
1. Try TPU QUIC send to primary target
2. If fail -> Try TPU to secondary target (if LeaderLookahead)
3. If fail -> Fall back to RPC sendTransaction
4. Return SendResult with actual method used
```

## SendResult

```rust
pub struct SendResult {
    pub signature: String,
    pub send_method: String,      // "tpu" or "rpc"
    pub latency_ms: f64,
    pub target_validator: String,
}
```

The `latency_ms` feeds back into the topology for future route optimization.

## TPU Address Resolution

Validator TPU addresses come from two sources:

1. **Cluster info** (`getClusterNodes` RPC): Returns TPU addresses for all validators
2. **Gossip network**: Real-time peer discovery (not yet implemented in REVM)

```rust
ValidatorEntry {
    pubkey: "Vote111...".into(),
    tpu_addr: Some("10.0.1.1:8004".into()),  // TPU UDP
    // QUIC port is typically tpu_addr + 6 (8004 -> 8009)
}
```

## Rate Limiting Considerations

Solana's QUIC implementation rate-limits by:

| Factor | Effect |
|---|---|
| Stake weight | Higher stake = more allowed streams |
| Connection age | Established connections get priority |
| IP reputation | Known good IPs face less throttling |

REVM's `StakeWeighted` routing strategy leverages this — routing through high-stake validators that are less likely to be rate-limited.
