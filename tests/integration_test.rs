use revm_core::aco::config::AcoConfig;
use revm_core::aco::colony::Colony;
use revm_core::aco::pheromone::PheromoneMatrix;
use revm_core::network::topology::{NetworkTopology, ValidatorEntry};
use revm_core::router::engine::RoutingEngine;
use revm_core::router::strategy::RoutingStrategy;
use revm_core::solana::leader::{LeaderScheduleEntry, LeaderTracker};

/// Build a realistic Solana-like topology with entry point + 8 validators.
fn build_mainnet_topology() -> NetworkTopology {
    let validators = vec![
        ValidatorEntry {
            pubkey: "Certusone111111111111111111111111111111111111".into(),
            stake_weight: 0.08,
            estimated_latency_ms: 6.0,
            is_leader: true,
            tpu_addr: Some("10.0.1.1:8004".into()),
        },
        ValidatorEntry {
            pubkey: "Everstake222222222222222222222222222222222222".into(),
            stake_weight: 0.06,
            estimated_latency_ms: 8.0,
            is_leader: false,
            tpu_addr: Some("10.0.2.1:8004".into()),
        },
        ValidatorEntry {
            pubkey: "Chorus33333333333333333333333333333333333333".into(),
            stake_weight: 0.05,
            estimated_latency_ms: 11.0,
            is_leader: false,
            tpu_addr: Some("10.0.3.1:8004".into()),
        },
        ValidatorEntry {
            pubkey: "Figment4444444444444444444444444444444444444".into(),
            stake_weight: 0.04,
            estimated_latency_ms: 7.0,
            is_leader: false,
            tpu_addr: Some("10.0.4.1:8004".into()),
        },
        ValidatorEntry {
            pubkey: "P2Pval55555555555555555555555555555555555555".into(),
            stake_weight: 0.03,
            estimated_latency_ms: 14.0,
            is_leader: false,
            tpu_addr: None,
        },
        ValidatorEntry {
            pubkey: "Staked66666666666666666666666666666666666666".into(),
            stake_weight: 0.07,
            estimated_latency_ms: 5.0,
            is_leader: false,
            tpu_addr: Some("10.0.6.1:8004".into()),
        },
        ValidatorEntry {
            pubkey: "Laine777777777777777777777777777777777777777".into(),
            stake_weight: 0.02,
            estimated_latency_ms: 9.0,
            is_leader: false,
            tpu_addr: None,
        },
        ValidatorEntry {
            pubkey: "Jump8888888888888888888888888888888888888888".into(),
            stake_weight: 0.09,
            estimated_latency_ms: 4.0,
            is_leader: false,
            tpu_addr: Some("10.0.8.1:8004".into()),
        },
    ];

    NetworkTopology::from_cluster_snapshot(validators, "rpc.mainnet-beta.solana.com")
}

#[test]
fn test_full_routing_pipeline() {
    let topo = build_mainnet_topology();
    let config = AcoConfig::mainnet();
    let mut colony = Colony::new(topo, config).unwrap();

    // Route from entry point (0) to leader validator (1)
    let result = colony.route(0, 1).unwrap();

    assert!(!result.path.is_empty());
    assert_eq!(*result.path.first().unwrap(), 0);
    assert_eq!(*result.path.last().unwrap(), 1);
    assert!(result.cost > 0.0);
    assert!(result.cost < 50.0); // should be well under 50ms
}

#[test]
fn test_routing_convergence_over_iterations() {
    let topo = build_mainnet_topology();
    let config = AcoConfig {
        ant_count: 64,
        max_iterations: 100,
        ..AcoConfig::mainnet()
    };
    let mut colony = Colony::new(topo, config).unwrap();

    let mut costs: Vec<f64> = Vec::new();
    for _ in 0..10 {
        let result = colony.route(0, 1).unwrap();
        costs.push(result.cost);
    }

    // Later iterations should converge to similar or better cost
    let first_half_avg: f64 = costs[..5].iter().sum::<f64>() / 5.0;
    let second_half_avg: f64 = costs[5..].iter().sum::<f64>() / 5.0;
    assert!(
        second_half_avg <= first_half_avg * 1.1,
        "Expected convergence: first_half={:.2}, second_half={:.2}",
        first_half_avg,
        second_half_avg
    );
}

#[test]
fn test_pheromone_reinforcement() {
    let topo = build_mainnet_topology();
    let config = AcoConfig::mainnet();
    let pheromone = PheromoneMatrix::new(topo.node_count(), &config);

    // Simulate multiple deposits on the optimal path
    let good_path = vec![0, 1]; // direct to leader
    for _ in 0..20 {
        pheromone.deposit_path(&good_path, 6.0, config.deposit_weight);
    }

    let bad_path = vec![0, 5, 1]; // indirect path
    for _ in 0..5 {
        pheromone.deposit_path(&bad_path, 15.0, config.deposit_weight);
    }

    // Good path should have significantly more pheromone
    let good_pheromone = pheromone.get(0, 1);
    let bad_pheromone = pheromone.get(0, 5);
    assert!(
        good_pheromone > bad_pheromone,
        "Expected good_path pheromone ({:.4}) > bad_path ({:.4})",
        good_pheromone,
        bad_pheromone
    );
}

#[test]
fn test_engine_with_leader_strategy() {
    let topo = build_mainnet_topology();
    let config = AcoConfig::mainnet();

    let mut engine = RoutingEngine::new(
        topo,
        config,
        RoutingStrategy::LeaderOnly,
    )
    .unwrap();

    let decision = engine.route_transaction(0).unwrap();
    assert!(decision.estimated_latency_ms > 0.0);
    assert!(decision.computation_time_us > 0);

    // Leader-only should route to node 1 (the leader in our topology)
    let target = *decision.primary_path.last().unwrap();
    assert_eq!(target, 1, "Expected route to leader node");
}

#[test]
fn test_engine_with_stake_weighted_strategy() {
    let topo = build_mainnet_topology();
    let config = AcoConfig::mainnet();

    let mut engine = RoutingEngine::new(
        topo,
        config,
        RoutingStrategy::StakeWeighted { top_n: 3 },
    )
    .unwrap();

    let decision = engine.route_transaction(0).unwrap();

    // Top 3 by stake: Jump(0.09), Certus(0.08), Staked(0.07) -> nodes 8, 1, 6
    assert!(decision.target_validators.len() <= 3);
    assert!(decision.estimated_latency_ms < 30.0);
}

#[test]
fn test_engine_full_colony_strategy() {
    let topo = build_mainnet_topology();
    let config = AcoConfig {
        ant_count: 32,
        max_iterations: 20,
        ..AcoConfig::mainnet()
    };

    let mut engine = RoutingEngine::new(
        topo,
        config,
        RoutingStrategy::FullColony,
    )
    .unwrap();

    let decision = engine.route_transaction(0).unwrap();
    assert!(decision.hop_count >= 1);
    assert!(decision.estimated_latency_ms > 0.0);
}

#[test]
fn test_leader_tracker_integration() {
    let mut tracker = LeaderTracker::new(432000);

    tracker.register_validator("Certusone111111111111111111111111111111111111".into(), 1);
    tracker.register_validator("Jump8888888888888888888888888888888888888888".into(), 8);

    let entries = vec![
        LeaderScheduleEntry {
            slot: 1000,
            leader_pubkey: "Certusone111111111111111111111111111111111111".into(),
        },
        LeaderScheduleEntry {
            slot: 1001,
            leader_pubkey: "Certusone111111111111111111111111111111111111".into(),
        },
        LeaderScheduleEntry {
            slot: 1002,
            leader_pubkey: "Jump8888888888888888888888888888888888888888".into(),
        },
        LeaderScheduleEntry {
            slot: 1003,
            leader_pubkey: "Jump8888888888888888888888888888888888888888".into(),
        },
    ];

    tracker.load_schedule(entries, 0);
    tracker.set_current_slot(1000);

    assert_eq!(tracker.current_leader(), Some(1));

    let lookahead = tracker.leaders_ahead(2);
    assert!(lookahead.contains(&1));
    assert!(lookahead.contains(&8));
}

#[test]
fn test_config_validation() {
    let config = AcoConfig::default();
    assert!(config.validate().is_ok());

    let bad_config = AcoConfig {
        evaporation_rate: 1.5, // invalid: must be in (0, 1)
        ..AcoConfig::default()
    };
    assert!(bad_config.validate().is_err());

    let bad_config2 = AcoConfig {
        pheromone_min: 10.0,
        pheromone_max: 5.0, // invalid: min >= max
        ..AcoConfig::default()
    };
    assert!(bad_config2.validate().is_err());
}

#[test]
fn test_topology_stale_edge_detection() {
    let mut topo = NetworkTopology::new(3);
    topo.add_edge(0, 1, 5.0);
    topo.add_edge(0, 2, 8.0);

    // All edges are fresh (just created), so nothing should be stale
    let stale = topo.stale_edges(1000);
    assert!(stale.is_empty());

    // With a very large threshold, edges created just now are stale
    // (now - last_measured >= 0, and we use a threshold that makes the check trivially true)
    std::thread::sleep(std::time::Duration::from_millis(2));
    let stale = topo.stale_edges(1);
    assert!(!stale.is_empty());
}
