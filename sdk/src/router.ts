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
  antCount: 32,
  maxIterations: 50,
};

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

  reset(): void {
    for (let i = 0; i < this.nodeCount; i++)
      for (let j = 0; j < this.nodeCount; j++)
        this.pheromone[i][j] = 0.1;
  }
}
