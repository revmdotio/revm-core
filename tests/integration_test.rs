use revm_core::aco::config::AcoConfig;
use revm_core::aco::colony::Colony;
use revm_core::network::topology::{NetworkTopology, ValidatorEntry};

fn build_mainnet_topology() -> NetworkTopology {
    let validators = vec![
        ValidatorEntry {
            pubkey: "Certusone111111111111111111111111111111111111".into(),
            stake_weight: 0.08, estimated_latency_ms: 6.0,
            is_leader: true, tpu_addr: Some("10.0.1.1:8004".into()),
        },
        ValidatorEntry {
            pubkey: "Everstake222222222222222222222222222222222222".into(),
            stake_weight: 0.06, estimated_latency_ms: 8.0,
            is_leader: false, tpu_addr: None,
        },
    ];
    NetworkTopology::from_cluster_snapshot(validators, "rpc.mainnet-beta.solana.com")
}

#[test]
fn test_full_routing_pipeline() {
    let topo = build_mainnet_topology();
    let config = AcoConfig::mainnet();
    let mut colony = Colony::new(topo, config).unwrap();
    let result = colony.route(0, 1).unwrap();
    assert!(!result.path.is_empty());
    assert_eq!(*result.path.first().unwrap(), 0);
    assert!(result.cost < 50.0);
}
