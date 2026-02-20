import { Connection, Transaction, VersionedTransaction } from '@solana/web3.js';
import { AcoRouter } from './router';
import { TransactionPayload, SendOptions, SendResult, ValidatorNode, ColonyMetrics } from './types';

export interface RevmClientConfig {
  rpcUrl: string;
  wsUrl?: string;
  acoConfig?: Partial<import('./router').AcoRouterConfig>;
}

export class RevmClient {
  private connection: Connection;
  private router: AcoRouter | null = null;
  private validators: ValidatorNode[] = [];
  private config: RevmClientConfig;
  private initialized = false;

  constructor(config: RevmClientConfig) {
    this.config = config;
    this.connection = new Connection(config.rpcUrl, {
      wsEndpoint: config.wsUrl,
      commitment: 'confirmed',
    });
  }

  isInitialized(): boolean { return this.initialized; }
}
