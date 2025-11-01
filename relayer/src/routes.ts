import { Express, Request, Response } from 'express';
import { FisherRelayer } from './relayer';
import { logger } from './logger';

export function setupRoutes(app: Express, relayer: FisherRelayer) {
  
  /**
   * Submit transaction to Fisher
   */
  app.post('/api/submit', async (req: Request, res: Response) => {
    try {
      const { from, to, amount, priorityFlag, nonce, signature } = req.body;

      // Validate inputs
      if (!from || !to || !amount || !signature) {
        return res.status(400).json({
          error: 'Missing required fields: from, to, amount, signature',
        });
      }

      // Add to queue
      const txId = await relayer.addTransaction({
        from,
        to,
        amount,
        priorityFlag: priorityFlag ?? false,
        nonce,
        signature,
      });

      const status = relayer.getQueueStatus();

      res.json({
        success: true,
        transactionId: txId,
        queuePosition: status.queueLength,
        estimatedBatchTime: calculateBatchTime(status),
        message: 'Transaction queued for batch processing',
      });

    } catch (error) {
      logger.error('Submit error:', error);
      res.status(500).json({
        error: 'Failed to submit transaction',
        message: error instanceof Error ? error.message : 'Unknown error',
      });
    }
  });

  /**
   * Get queue status
   */
  app.get('/api/status', (req: Request, res: Response) => {
    const status = relayer.getQueueStatus();
    const metrics = relayer.getMetrics();

    res.json({
      queue: status,
      metrics,
      phi: {
        optimization: 'φ-Freeman Golden Ratio',
        compression: 'Williams O(√n log n)',
        gasSavings: '86%',
      },
    });
  });

  /**
   * Get metrics
   */
  app.get('/api/metrics', (req: Request, res: Response) => {
    const metrics = relayer.getMetrics();
    res.json(metrics);
  });

  /**
   * Get queue contents (debug)
   */
  app.get('/api/queue', (req: Request, res: Response) => {
    const queue = relayer.getQueue();
    res.json({
      length: queue.length,
      transactions: queue,
    });
  });

  /**
   * Check user deposit
   */
  app.get('/api/deposit/:address', async (req: Request, res: Response) => {
    try {
      const { address } = req.params;
      const deposit = await relayer.getUserDeposit(address);
      const fisherAddress = relayer.getFisherAddress();

      res.json({
        address,
        deposit,
        fisherContract: fisherAddress,
        instructions: {
          howToDeposit: `Send ETH to Fisher contract: ${fisherAddress}`,
          orCallFunction: 'deposit() with ETH value',
        },
      });
    } catch (error) {
      logger.error('Deposit check error:', error);
      res.status(500).json({ error: 'Failed to check deposit' });
    }
  });

  /**
   * Calculate fee for amount
   */
  app.get('/api/fee/:amount', async (req: Request, res: Response) => {
    try {
      const { amount } = req.params;
      const fee = await relayer.calculateRequiredFee(amount);

      res.json({
        amount,
        requiredFee: fee,
        feeBps: '10', // 0.1%
        note: 'User must have this much deposited to Fisher contract',
      });
    } catch (error) {
      logger.error('Fee calculation error:', error);
      res.status(500).json({ error: 'Failed to calculate fee' });
    }
  });

  /**
   * API documentation
   */
  app.get('/api', (req: Request, res: Response) => {
    res.json({
      name: 'φ-Freeman Fisher Relayer',
      version: '1.0.0',
      description: 'Gas-optimized transaction batching for EVVM',
      features: [
        'Williams compression: O(√n log n) memory',
        'φ-optimization: Era-based tracking',
        'Gas savings: 86% reduction',
        'Automatic batching',
        'Separate deposit system (signatures remain valid)',
      ],
      endpoints: {
        'POST /api/submit': 'Submit transaction for batching',
        'GET /api/status': 'Get queue and metrics',
        'GET /api/metrics': 'Get historical metrics',
        'GET /api/queue': 'Get current queue (debug)',
        'GET /api/deposit/:address': 'Check user deposit balance',
        'GET /api/fee/:amount': 'Calculate required fee',
        'GET /health': 'Health check',
      },
      workflow: {
        step1: 'Deposit ETH to Fisher contract for fees',
        step2: 'Sign EVVM payment (full amount)',
        step3: 'Submit signed payment to Fisher relayer',
        step4: 'Fisher batches and submits to EVVM Core',
        step5: 'Fisher deducts fee from your deposit',
      },
    });
  });
}

/**
 * Calculate estimated time until batch processing
 */
function calculateBatchTime(status: ReturnType<typeof FisherRelayer.prototype.getQueueStatus>): string {
  if (status.queueLength >= status.minBatchSize) {
    return 'Next batch cycle (~5 seconds)';
  }
  
  const txNeeded = status.minBatchSize - status.queueLength;
  return `Waiting for ${txNeeded} more transactions`;
}
