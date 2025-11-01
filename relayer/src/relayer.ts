import { ethers } from 'ethers';
import { logger } from './logger';

export interface Transaction {
  id: string;
  from: string;
  to: string;
  amount: string;
  priorityFlag: boolean;
  nonce?: number;
  signature: string;
  timestamp: number;
}

export interface BatchMetrics {
  batchId: number;
  transactionCount: number;
  gasUsed: bigint;
  gasSaved: bigint;
  savingsPercent: number;
  timestamp: number;
}

export interface RelayerConfig {
  rpcUrl: string;
  privateKey: string;
  fisherAddress: string;
  evvmCoreAddress: string;
  minBatchSize: number;
  maxBatchSize: number;
  batchInterval: number;
}

export class FisherRelayer {
  private provider: ethers.JsonRpcProvider;
  private wallet: ethers.Wallet;
  private fisher: ethers.Contract;
  private transactionQueue: Transaction[] = [];
  private processingQueue = false;
  private batchTimer: NodeJS.Timeout | null = null;
  private metrics: BatchMetrics[] = [];
  
  public readonly minBatchSize: number;
  public readonly maxBatchSize: number;
  public readonly batchInterval: number;

  // FisherProduction ABI (minimal)
  private readonly FISHER_ABI = [
    'function submitBatchOptimized((address from, address to, uint256 amount, bool priorityFlag, uint256 nonce)[] payments, bytes[] signatures) returns (bool[])',
    'function calculateChunkSize(uint256 batchSize) view returns (uint256)',
    'function estimateGas(uint256 batchSize) view returns (uint256, uint256)',
    'function calculateFee(uint256 amount) view returns (uint256)',
    'function deposits(address user) view returns (uint256)',
    'function deposit() payable',
    'function withdraw(uint256 amount)',
    'function batchCounter() view returns (uint256)',
    'function relayerFeeBps() view returns (uint256)',
    'event BatchSubmitted(uint256 indexed batchId, uint256 operationCount, uint256 gasUsed, uint256 gasSaved, uint256 feesCollected, uint256 timestamp)',
    'event Deposited(address indexed user, uint256 amount)',
    'event FeeCharged(address indexed user, uint256 amount)',
  ];

  constructor(config: RelayerConfig) {
    this.minBatchSize = config.minBatchSize;
    this.maxBatchSize = config.maxBatchSize;
    this.batchInterval = config.batchInterval;

    // Setup provider and wallet
    this.provider = new ethers.JsonRpcProvider(config.rpcUrl);
    this.wallet = new ethers.Wallet(config.privateKey, this.provider);
    
    // Setup Fisher contract
    this.fisher = new ethers.Contract(
      config.fisherAddress,
      this.FISHER_ABI,
      this.wallet
    );

    logger.info('✅ Fisher Relayer initialized');
    logger.info(`📍 Fisher: ${config.fisherAddress}`);
    logger.info(`📍 EVVM Core: ${config.evvmCoreAddress}`);
  }

  /**
   * Start batch processing timer
   */
  start(): void {
    logger.info('🚀 Starting batch processor...');
    
    this.batchTimer = setInterval(() => {
      this.processBatch();
    }, this.batchInterval);

    // Listen for BatchSubmitted events
    this.fisher.on('BatchSubmitted', (batchId, opCount, gasUsed, gasSaved, feesCollected, timestamp) => {
      const savingsPercent = Number(gasSaved) / (Number(gasUsed) + Number(gasSaved)) * 100;
      
      logger.info(`📦 Batch ${batchId} submitted:`);
      logger.info(`   ✅ ${opCount} transactions`);
      logger.info(`   ⚡ ${gasUsed} gas used`);
      logger.info(`   💰 ${gasSaved} gas saved (${savingsPercent.toFixed(2)}%)`);
      logger.info(`   💵 ${feesCollected} fees collected`);

      this.metrics.push({
        batchId: Number(batchId),
        transactionCount: Number(opCount),
        gasUsed: BigInt(gasUsed),
        gasSaved: BigInt(gasSaved),
        savingsPercent,
        timestamp: Number(timestamp),
      });
    });
  }

  /**
   * Stop batch processing
   */
  stop(): void {
    if (this.batchTimer) {
      clearInterval(this.batchTimer);
      this.batchTimer = null;
    }
    logger.info('🛑 Batch processor stopped');
  }

  /**
   * Add transaction to queue
   */
  async addTransaction(tx: Omit<Transaction, 'id' | 'timestamp'>): Promise<string> {
    // Check user has sufficient deposit for fee
    const amount = ethers.parseEther(tx.amount);
    const requiredFee = await this.fisher.calculateFee(amount);
    const userDeposit = await this.fisher.deposits(tx.from);
    
    if (userDeposit < requiredFee) {
      logger.warn(`❌ Insufficient deposit for ${tx.from}`);
      logger.warn(`   Required: ${ethers.formatEther(requiredFee)} ETH`);
      logger.warn(`   Has: ${ethers.formatEther(userDeposit)} ETH`);
      throw new Error(`Insufficient deposit. Required: ${ethers.formatEther(requiredFee)} ETH`);
    }
    
    const id = ethers.id(`${tx.from}-${tx.to}-${Date.now()}`);
    
    const transaction: Transaction = {
      ...tx,
      id,
      timestamp: Date.now(),
    };

    this.transactionQueue.push(transaction);
    
    logger.info(`📥 Transaction queued: ${id.slice(0, 10)}...`);
    logger.info(`   From: ${tx.from}`);
    logger.info(`   To: ${tx.to}`);
    logger.info(`   Amount: ${tx.amount}`);
    logger.info(`   Fee: ${ethers.formatEther(requiredFee)} ETH`);
    logger.info(`   Queue size: ${this.transactionQueue.length}`);

    // Process immediately if queue is full
    if (this.transactionQueue.length >= this.maxBatchSize) {
      logger.info('📦 Queue full, processing immediately...');
      setImmediate(() => this.processBatch());
    }

    return id;
  }

  /**
   * Process batch using Williams compression
   */
  private async processBatch(): Promise<void> {
    if (this.processingQueue || this.transactionQueue.length < this.minBatchSize) {
      return;
    }

    this.processingQueue = true;

    try {
      // Take batch from queue (up to maxBatchSize)
      const batchSize = Math.min(this.transactionQueue.length, this.maxBatchSize);
      const batch = this.transactionQueue.splice(0, batchSize);

      logger.info(`🔨 Processing batch of ${batch.length} transactions...`);

      // Calculate Williams chunk size
      const chunkSize = await this.fisher.calculateChunkSize(batch.length);
      logger.info(`   📊 Williams chunk size: ${chunkSize}`);

      // Prepare payments and signatures
      const payments = batch.map(tx => ({
        from: tx.from,
        to: tx.to,
        amount: ethers.parseEther(tx.amount),
        priorityFlag: tx.priorityFlag,
        nonce: tx.nonce || 0,
      }));

      const signatures = batch.map(tx => tx.signature);

      // Get gas estimate
      const [estimatedGas, estimatedSavings] = await this.fisher.estimateGas(batch.length);
      logger.info(`   ⚡ Estimated gas: ${estimatedGas}`);
      logger.info(`   💰 Estimated savings: ${estimatedSavings}`);

      // Submit batch
      logger.info('   📤 Submitting to Fisher contract...');
      const tx = await this.fisher.submitBatchOptimized(payments, signatures);
      
      logger.info(`   ⏳ Transaction sent: ${tx.hash}`);
      const receipt = await tx.wait();
      
      logger.info(`   ✅ Batch processed in block ${receipt.blockNumber}`);
      logger.info(`   ⛽ Actual gas used: ${receipt.gasUsed}`);

    } catch (error) {
      logger.error('❌ Batch processing failed:', error);
      // Re-queue transactions on failure
      // (in production, implement more sophisticated retry logic)
    } finally {
      this.processingQueue = false;
    }
  }

  /**
   * Get current queue status
   */
  getQueueStatus() {
    return {
      queueLength: this.transactionQueue.length,
      processing: this.processingQueue,
      minBatchSize: this.minBatchSize,
      maxBatchSize: this.maxBatchSize,
      batchInterval: this.batchInterval,
    };
  }

  /**
   * Get metrics
   */
  getMetrics() {
    const totalBatches = this.metrics.length;
    const totalTransactions = this.metrics.reduce((sum, m) => sum + m.transactionCount, 0);
    const totalGasUsed = this.metrics.reduce((sum, m) => sum + m.gasUsed, 0n);
    const totalGasSaved = this.metrics.reduce((sum, m) => sum + m.gasSaved, 0n);
    const avgSavings = totalBatches > 0
      ? this.metrics.reduce((sum, m) => sum + m.savingsPercent, 0) / totalBatches
      : 0;

    return {
      totalBatches,
      totalTransactions,
      totalGasUsed: totalGasUsed.toString(),
      totalGasSaved: totalGasSaved.toString(),
      averageSavingsPercent: avgSavings,
      recentBatches: this.metrics.slice(-10),
    };
  }

  /**
   * Get queue contents (for debugging)
   */
  getQueue() {
    return this.transactionQueue;
  }

  /**
   * Check user deposit balance
   */
  async getUserDeposit(address: string): Promise<string> {
    const deposit = await this.fisher.deposits(address);
    return ethers.formatEther(deposit);
  }

  /**
   * Calculate required fee for amount
   */
  async calculateRequiredFee(amount: string): Promise<string> {
    const amountWei = ethers.parseEther(amount);
    const fee = await this.fisher.calculateFee(amountWei);
    return ethers.formatEther(fee);
  }

  /**
   * Get Fisher contract address for deposits
   */
  getFisherAddress(): string {
    return this.fisher.target as string;
  }
}
