use criterion::{black_box, criterion_group, criterion_main, Criterion};

use revm_core::aco::colony::Colony;
use revm_core::aco::config::AcoConfig;
use revm_core::aco::pheromone::PheromoneMatrix;
use revm_core::network::topology::{NetworkTopology, ValidatorEntry};
use revm_core::router::engine::RoutingEngine;
use revm_core::router::strategy::RoutingStrategy;

fn build_topology(num_validators: usize) -> NetworkTopology {
    let validators: Vec<ValidatorEntry> = (0..num_validators)
        .map(|i| ValidatorEntry {
            pubkey: format!("Validator{:0>40}", i),
            stake_weight: 0.01 + (i as f64 * 0.005),
            estimated_latency_ms: 3.0 + (i as f64 * 1.5),
            is_leader: i == 0,
            tpu_addr: Some(format!("10.0.{}.1:8004", i)),
        })
        .collect();

    NetworkTopology::from_cluster_snapshot(validators, "benchmark-entry")
}

fn bench_colony_route(c: &mut Criterion) {
    let mut group = c.benchmark_group("colony_routing");

    for &size in &[8, 16, 32, 64] {
        group.bench_function(format!("route_{}_validators", size), |b| {
            let topo = build_topology(size);
            let config = AcoConfig {
                ant_count: 32,
                max_iterations: 20,
                ..AcoConfig::mainnet()
            };
            let mut colony = Colony::new(topo, config).unwrap();

            b.iter(|| {
                let result = colony.route(black_box(0), black_box(1));
                black_box(result)
            });
        });
    }

    group.finish();
}

fn bench_pheromone_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("pheromone");
    let config = AcoConfig::mainnet();

    group.bench_function("evaporate_64x64", |b| {
        let matrix = PheromoneMatrix::new(64, &config);
        b.iter(|| {
            matrix.evaporate();
            black_box(&matrix);
        });
    });

    group.bench_function("deposit_path_10_hops", |b| {
        let matrix = PheromoneMatrix::new(64, &config);
        let path: Vec<usize> = (0..10).collect();
        b.iter(|| {
            matrix.deposit_path(black_box(&path), black_box(5.0), black_box(1.0));
        });
    });

    group.bench_function("snapshot_64x64", |b| {
        let matrix = PheromoneMatrix::new(64, &config);
        b.iter(|| {
            let snap = matrix.snapshot();
            black_box(snap);
        });
    });

    group.finish();
}

fn bench_routing_engine(c: &mut Criterion) {
    let mut group = c.benchmark_group("routing_engine");

    group.bench_function("leader_only_32_validators", |b| {
        let topo = build_topology(32);
        let config = AcoConfig {
            ant_count: 32,
            max_iterations: 15,
            ..AcoConfig::mainnet()
        };
        let mut engine =
            RoutingEngine::new(topo, config, RoutingStrategy::LeaderOnly).unwrap();

        b.iter(|| {
            let decision = engine.route_transaction(black_box(0));
            black_box(decision)
        });
    });

    group.bench_function("full_colony_32_validators", |b| {
        let topo = build_topology(32);
        let config = AcoConfig {
            ant_count: 16,
            max_iterations: 10,
            ..AcoConfig::mainnet()
        };
        let mut engine =
            RoutingEngine::new(topo, config, RoutingStrategy::FullColony).unwrap();

        b.iter(|| {
            let decision = engine.route_transaction(black_box(0));
            black_box(decision)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_colony_route,
    bench_pheromone_operations,
    bench_routing_engine
);
criterion_main!(benches);
