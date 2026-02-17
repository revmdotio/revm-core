import { Connection, Transaction, VersionedTransaction } from '@solana/web3.js';

export interface TransactionPayload {
  transaction: Transaction | VersionedTransaction;
  priorityFee?: number;
  skipPreflight?: boolean;
  maxRetries?: number;
}

export interface SendOptions {
  strategy?: 'leader-only' | 'leader-lookahead' | 'stake-weighted' | 'full-colony';
  slotsAhead?: number;
  topN?: number;
  timeout?: number;
}

export interface SendResult {
  signature: string;
  targetValidator: string;
  sendLatencyMs: number;
  hopCount: number;
  slot: number;
  confirmed: boolean;
}

export interface ValidatorNode {
  pubkey: string;
  stakeWeight: number;
  latencyMs: number;
  isLeader: boolean;
  tpuAddr?: string;
}

export interface PheromoneEdge {
  from: number;
  to: number;
  intensity: number;
}

export interface ColonyMetrics {
  totalTransactions: number;
  successfulRoutes: number;
  failedRoutes: number;
  avgLatencyMs: number;
  avgHops: number;
  p99LatencyMs: number;
}
