import express from 'express';
import cors from 'cors';
import dotenv from 'dotenv';
import { FisherRelayer } from './relayer';
import { setupRoutes } from './routes';
import { logger } from './logger';

dotenv.config();

const app = express();
const PORT = process.env.PORT || 3001;

// Middleware
app.use(cors());
app.use(express.json());

// Initialize Fisher Relayer
const relayer = new FisherRelayer({
  rpcUrl: process.env.SEPOLIA_RPC_URL || 'https://rpc.sepolia.org',
  privateKey: process.env.PRIVATE_KEY || '',
  fisherAddress: process.env.FISHER_ADDRESS || '',
  evvmCoreAddress: process.env.EVVM_CORE || '0xF817e9ad82B4a19F00dA7A248D9e556Ba96e6366',
  minBatchSize: parseInt(process.env.MIN_BATCH_SIZE || '10'),
  maxBatchSize: parseInt(process.env.MAX_BATCH_SIZE || '1000'),
  batchInterval: parseInt(process.env.BATCH_INTERVAL || '5000'), // 5 seconds
});

// Setup routes
setupRoutes(app, relayer);

// Health check
app.get('/health', (req, res) => {
  res.json({
    status: 'ok',
    uptime: process.uptime(),
    timestamp: Date.now(),
  });
});

// Start server
app.listen(PORT, () => {
  logger.info(`🚀 φ-Freeman Fisher Relayer running on port ${PORT}`);
  logger.info(`📊 Min batch size: ${relayer.minBatchSize}`);
  logger.info(`📊 Max batch size: ${relayer.maxBatchSize}`);
  logger.info(`⏱️  Batch interval: ${relayer.batchInterval}ms`);
  
  // Start batch processing
  relayer.start();
});

// Graceful shutdown
process.on('SIGTERM', () => {
  logger.info('SIGTERM received, shutting down gracefully');
  relayer.stop();
  process.exit(0);
});

process.on('SIGINT', () => {
  logger.info('SIGINT received, shutting down gracefully');
  relayer.stop();
  process.exit(0);
});
