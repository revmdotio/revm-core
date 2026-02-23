import { AcoRouter } from '../src/router';
import { ValidatorNode } from '../src/types';

describe('AcoRouter', () => {
  it('finds a direct path in simple topology', () => {
    const router = new AcoRouter(3, { antCount: 16, maxIterations: 30 });
    router.setEdge(0, 1, 5);
    router.setEdge(1, 2, 3);

    const result = router.route(0, 2);
    expect(result).not.toBeNull();
    expect(result!.path).toEqual([0, 1, 2]);
    expect(result!.cost).toBeCloseTo(8, 1);
    expect(result!.hopCount).toBe(2);
  });

  it('returns null when no path exists', () => {
    const router = new AcoRouter(3, { antCount: 8, maxIterations: 10 });
    router.setEdge(0, 1, 5);
    // No edge from 1 to 2

    const result = router.route(0, 2);
    expect(result).toBeNull();
  });

  it('selects lower-cost path with pheromone reinforcement', () => {
    const router = new AcoRouter(4, { antCount: 32, maxIterations: 50 });

    // Diamond: 0->1->3 (cost 9) vs 0->2->3 (cost 15)
    router.setEdge(0, 1, 5);
    router.setEdge(0, 2, 10);
    router.setEdge(1, 3, 4);
    router.setEdge(2, 3, 5);

    // Run multiple times to build pheromone
    let lastResult = null;
    for (let i = 0; i < 5; i++) {
      lastResult = router.route(0, 3);
    }

    expect(lastResult).not.toBeNull();
    // Should converge to the cheaper path (0->1->3, cost 9)
    expect(lastResult!.cost).toBeLessThanOrEqual(12);
  });

  it('loads topology from validator list', () => {
    const validators: ValidatorNode[] = [
      { pubkey: 'val1', stakeWeight: 0.05, latencyMs: 6, isLeader: true },
      { pubkey: 'val2', stakeWeight: 0.03, latencyMs: 10, isLeader: false },
      { pubkey: 'val3', stakeWeight: 0.08, latencyMs: 4, isLeader: false },
    ];

    const router = new AcoRouter(4, { antCount: 16, maxIterations: 20 });
    router.loadTopology(validators);

    // Route from entry (0) to validator 3 (node 3)
    const result = router.route(0, 3);
    expect(result).not.toBeNull();
    expect(result!.path[0]).toBe(0);
    expect(result!.path[result!.path.length - 1]).toBe(3);
  });

  it('generates pheromone snapshot', () => {
    const router = new AcoRouter(4, { antCount: 8, maxIterations: 10 });
    router.setEdge(0, 1, 5);
    router.setEdge(1, 2, 3);

    router.route(0, 2);
    const snapshot = router.getPheromoneSnapshot();

    expect(snapshot.length).toBeGreaterThan(0);
    expect(snapshot[0]).toHaveProperty('from');
    expect(snapshot[0]).toHaveProperty('to');
    expect(snapshot[0]).toHaveProperty('intensity');
  });

  it('resets pheromone state', () => {
    const router = new AcoRouter(3, { antCount: 16, maxIterations: 20 });
    router.setEdge(0, 1, 5);
    router.setEdge(1, 2, 3);

    // Build up pheromone
    for (let i = 0; i < 5; i++) {
      router.route(0, 2);
    }

    const before = router.getPheromoneSnapshot();
    router.reset();
    const after = router.getPheromoneSnapshot();

    // After reset, all pheromone should be at initial value (0.1)
    const maxBefore = Math.max(...before.map((e) => e.intensity));
    const maxAfter = Math.max(...after.map((e) => e.intensity));
    expect(maxAfter).toBeLessThan(maxBefore);
  });
});
