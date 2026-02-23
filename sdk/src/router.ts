import { ValidatorNode, PheromoneEdge } from './types';

export interface AcoRouterConfig {
  alpha: number;
  beta: number;
  evaporationRate: number;
  antCount: number;
  maxIterations: number;
}

export interface RouteResult {
  path: number[];
  cost: number;
  hopCount: number;
  iterationsUsed: number;
}

const DEFAULT_CONFIG: AcoRouterConfig = {
  alpha: 1.2,
  beta: 3.0,
  evaporationRate: 0.25,
  antCount: 32, // default colony size
  maxIterations: 50,
};

/**
 * Client-side ACO router for computing optimal validator paths.
 *
 * This is a lightweight TypeScript implementation of the core Rust ACO
 * algorithm, designed for client-side route preview and fallback routing
 * when the Rust backend is unavailable.
 */
export class AcoRouter {
  private config: AcoRouterConfig;
  private pheromone: number[][];
  private latency: number[][];
  private nodeCount: number;

  constructor(nodeCount: number, config?: Partial<AcoRouterConfig>) {
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.nodeCount = nodeCount;
    this.pheromone = Array.from({ length: nodeCount }, () =>
      new Array(nodeCount).fill(0.1)
    );
    this.latency = Array.from({ length: nodeCount }, () =>
      new Array(nodeCount).fill(Infinity)
    );
  }

  setEdge(from: number, to: number, latencyMs: number): void {
    this.latency[from][to] = latencyMs;
  }

  setBidirectionalEdge(a: number, b: number, latencyMs: number): void {
    this.latency[a][b] = latencyMs;
    this.latency[b][a] = latencyMs;
  }

  loadTopology(validators: ValidatorNode[]): void {
    // Node 0 = entry point, validators are 1..N
    for (let i = 0; i < validators.length; i++) {
      const nodeId = i + 1;
      this.setEdge(0, nodeId, validators[i].latencyMs);

      for (let j = 0; j < validators.length; j++) {
        if (i !== j) {
          const interLatency =
            (validators[i].latencyMs + validators[j].latencyMs) * 0.3;
          this.setEdge(nodeId, j + 1, interLatency);
        }
      }
    }
  }

  route(source: number, destination: number): RouteResult | null {
    let bestPath: number[] | null = null;
    let bestCost = Infinity;
    let convergedAt = this.config.maxIterations;

    for (let iter = 0; iter < this.config.maxIterations; iter++) {
      let roundBestPath: number[] | null = null;
      let roundBestCost = Infinity;

      for (let a = 0; a < this.config.antCount; a++) {
        const result = this.antWalk(source, destination);
        if (result && result.cost < roundBestCost) {
          roundBestCost = result.cost;
          roundBestPath = result.path;
        }
      }

      this.evaporate();

      if (roundBestPath) {
        this.depositPath(roundBestPath, roundBestCost);
        if (roundBestCost < bestCost) {
          bestCost = roundBestCost;
          bestPath = roundBestPath;
        }
      }

      if (bestPath && iter > 10) {
        if (Math.abs(bestCost - roundBestCost) < 1e-9) {
          convergedAt = iter + 1;
          break;
        }
      }
    }

    if (!bestPath) return null;

    return {
      path: bestPath,
      cost: bestCost,
      hopCount: bestPath.length - 1,
      iterationsUsed: convergedAt,
    };
  }

  private antWalk(
    source: number,
    destination: number
  ): { path: number[]; cost: number } | null {
    const visited = new Set<number>();
    const path = [source];
    visited.add(source);
    let current = source;
    let cost = 0;

    while (current !== destination) {
      const candidates: number[] = [];
      for (let j = 0; j < this.nodeCount; j++) {
        if (!visited.has(j) && this.latency[current][j] < Infinity) {
          candidates.push(j);
        }
      }

      if (candidates.length === 0) return null;

      const probabilities: number[] = [];
      let total = 0;

      for (const next of candidates) {
        const tau = Math.pow(this.pheromone[current][next], this.config.alpha);
        const eta =
          this.latency[current][next] > 0
            ? Math.pow(1.0 / this.latency[current][next], this.config.beta)
            : 1.0;
        const score = tau * eta;
        probabilities.push(score);
        total += score;
      }

      if (total <= 0) return null;

      const threshold = Math.random() * total;
      let cumulative = 0;
      let selected = candidates[0];

      for (let i = 0; i < candidates.length; i++) {
        cumulative += probabilities[i];
        if (cumulative >= threshold) {
          selected = candidates[i];
          break;
        }
      }

      cost += this.latency[current][selected];
      path.push(selected);
      visited.add(selected);
      current = selected;
    }

    return { path, cost };
  }

  private evaporate(): void {
    const factor = 1 - this.config.evaporationRate;
    for (let i = 0; i < this.nodeCount; i++) {
      for (let j = 0; j < this.nodeCount; j++) {
        this.pheromone[i][j] = Math.max(this.pheromone[i][j] * factor, 0.001);
      }
    }
  }

  private depositPath(path: number[], cost: number): void {
    if (path.length < 2 || cost <= 0) return;
    const amount = 1.0 / cost;
    for (let i = 0; i < path.length - 1; i++) {
      this.pheromone[path[i]][path[i + 1]] = Math.min(
        this.pheromone[path[i]][path[i + 1]] + amount,
        10.0
      );
    }
  }

  getPheromoneSnapshot(): PheromoneEdge[] {
    const edges: PheromoneEdge[] = [];
    for (let i = 0; i < this.nodeCount; i++) {
      for (let j = 0; j < this.nodeCount; j++) {
        if (this.pheromone[i][j] > 0.002) {
          edges.push({ from: i, to: j, intensity: this.pheromone[i][j] });
        }
      }
    }
    return edges;
  }

  reset(): void {
    for (let i = 0; i < this.nodeCount; i++) {
      for (let j = 0; j < this.nodeCount; j++) {
        this.pheromone[i][j] = 0.1;
      }
    }
  }
}
