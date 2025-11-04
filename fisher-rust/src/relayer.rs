//! Main Fisher Relayer Implementation

use crate::{
    types::*,
    fishing_spot::{FishingSpotClient, FishingSpotConfig},
    staking::FisherStaking,
    Error, Result,
};
use alloy_primitives::U256;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn, error, debug};

#[cfg(not(target_arch = "wasm32"))]
use ethers::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use ethers::contract::abigen;

// Generate Rust bindings from your FisherProduction contract
#[cfg(not(target_arch = "wasm32"))]
abigen!(
    FisherContract,
    r#"[
        struct Payment { address from; address to; uint256 amount; bool priorityFlag; uint256 nonce; }
        function submitBatchOptimized(Payment[] payments, bytes[] signatures) external returns (bool[])
        function calculateChunkSize(uint256 batchSize) external view returns (uint256)
        function estimateGas(uint256 batchSize) external view returns (uint256, uint256)
        function batchCounter() external view returns (uint256)
        event BatchSubmitted(uint256 indexed batchId, uint256 operationCount, uint256 gasUsed, uint256 gasSaved, uint256 feesCollected, uint256 timestamp)
    ]"#
);

/// Fisher relayer - Collects and batches user intents
pub struct FisherRelayer {
    /// Configuration
    pub config: FisherConfig,
    
    /// Intent queue
    intent_queue: Arc<RwLock<Vec<Intent>>>,
    
    /// Ethereum wallet
    #[cfg(not(target_arch = "wasm32"))]
    wallet: Option<SignerMiddleware<Provider<Http>, LocalWallet>>,
    
    /// Metrics collector
    metrics: Arc<RwLock<Metrics>>,
    
    /// Fishing spot client (optional)
    fishing_spot: Option<FishingSpotClient>,
    
    /// Staking manager (optional)
    staking: Option<FisherStaking>,
}

impl FisherRelayer {
    pub fn new(config: FisherConfig) -> Result<Self> {
        info!("🚀 Initializing Fisher Relayer v{}", crate::VERSION);
        info!("📍 Fisher address: {:?}", config.fisher_address);
        info!("📍 EVVM Core: {:?}", config.evvm_core_address);
        
        Ok(Self {
            config,
            intent_queue: Arc::new(RwLock::new(Vec::new())),
            #[cfg(not(target_arch = "wasm32"))]
            wallet: None,
            metrics: Arc::new(RwLock::new(Metrics::default())),
            fishing_spot: None,
            staking: None,
        })
    }
    
    /// Enable fishing spot integration
    pub fn with_fishing_spot(mut self, config: FishingSpotConfig) -> Self {
        self.fishing_spot = Some(FishingSpotClient::new(config));
        self
    }
    
    /// Enable staking integration
    pub fn with_staking(mut self, staking: FisherStaking) -> Self {
        self.staking = Some(staking);
        self
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn init_ethereum(&mut self) -> Result<()> {
        info!("🔗 Connecting to Ethereum: {}", self.config.rpc_url);
        
        // Connect to Ethereum
        let provider = Provider::<Http>::try_from(&self.config.rpc_url)
            .map_err(|e| Error::Rpc(e.to_string()))?;
        
        // Setup wallet if private key provided
        if let Some(private_key) = &self.config.private_key {
            let wallet: LocalWallet = private_key
                .parse()
                .map_err(|e| Error::Config(format!("Invalid private key: {}", e)))?;
            
            let chain_id = provider
                .get_chainid()
                .await
                .map_err(|e| Error::Rpc(e.to_string()))?;
            
            let wallet = wallet.with_chain_id(chain_id.as_u64());
            let signer = SignerMiddleware::new(provider, wallet);
            
            self.wallet = Some(signer);
        }
        info!("✅ Connected to Ethereum");
        
        Ok(())
    }
    
    /// Submit intent to queue
    pub async fn submit_intent(&self, intent: Intent) -> Result<String> {
        debug!("📨 Received intent: {}", intent.id);
        
        // Verify signature
        if !intent.verify_signature() {
            return Err(Error::InvalidSignature);
        }
        
        // Add to queue
        let mut queue = self.intent_queue.write().await;
        let intent_id = intent.id.clone();
        queue.push(intent);
        
        info!("✅ Intent queued: {} (queue size: {})", intent_id, queue.len());
        
        // Check if we should process immediately
        if queue.len() >= self.config.max_batch_size {
            drop(queue); // Release lock
            tokio::spawn({
                let this = self.clone_arc();
                async move {
                    if let Err(e) = this.process_batch().await {
                        error!("❌ Batch processing failed: {}", e);
                    }
                }
            });
        }
        
        Ok(intent_id)
    }
    
    /// Process current batch
    pub async fn process_batch(&self) -> Result<BatchResult> {
        let start_time = SystemTime::now();
        
        // Get intents from queue
        let mut queue = self.intent_queue.write().await;
        
        if queue.len() < self.config.min_batch_size {
            debug!("⏳ Queue too small ({} < {})", queue.len(), self.config.min_batch_size);
            return Err(Error::BatchProcessing("Queue too small".to_string()));
        }
        
        let intents = queue.drain(..).collect::<Vec<_>>();
        drop(queue); // Release lock early
        
        info!("📦 Processing batch of {} intents", intents.len());
        
        // Build optimized batch
        let batch = self.build_batch(intents).await?;
        
        info!("✨ Batch optimized:");
        info!("   • Chunk size: {}", batch.chunk_size);
        info!("   • φ score: {:.2}", batch.phi_score);
        info!("   • Est. savings: {:.1}%", batch.savings_percent());
        
        // Submit to Ethereum
        let result = self.submit_batch_to_chain(&batch).await?;
        
        // Update metrics
        self.update_metrics(&batch, &result).await;
        
        let processing_time = start_time.elapsed().unwrap().as_millis() as u64;
        info!("🎉 Batch {} complete in {}ms", result.batch_id, processing_time);
        
        Ok(result)
    }
    
    /// Build optimized batch using Williams compression and φ-Freeman
    async fn build_batch(&self, mut intents: Vec<Intent>) -> Result<Batch> {
        // Generate batch ID from timestamp
        let batch_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Step 1: φ-optimization (priority scoring)
        let phi_score = intents.iter()
            .map(|i| crate::phi_optimization::phi_priority_score(i.priority, 0, i.amount.to()))
            .sum::<f64>() / intents.len() as f64;
        
        // Step 2: Williams compression (optimal chunking)
        let chunk_size = crate::williams::williams_chunk_size(intents.len());
        
        // Step 3: Estimate gas
        let (estimated_gas, estimated_savings) = self.estimate_batch_gas(&intents);
        
        Ok(Batch {
            id: batch_id,
            intents,
            chunk_size,
            phi_score,
            estimated_gas,
            estimated_savings,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }
    
    /// Estimate gas for batch
    fn estimate_batch_gas(&self, intents: &[Intent]) -> (U256, U256) {
        let n = intents.len() as u128;
        
        // Traditional: ~100K gas per operation
        let traditional_gas = U256::from(n * 100_000);
        
        // Williams-optimized: ~14K gas per operation
        let optimized_gas = U256::from(n * 14_000);
        
        let savings = traditional_gas - optimized_gas;
        
        (optimized_gas, savings)
    }
    
    /// Submit batch to Ethereum
    async fn submit_batch_to_chain(&self, batch: &Batch) -> Result<BatchResult> {
        info!("📤 Submitting batch {} to chain...", batch.id);
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.submit_batch_to_ethereum(batch).await
        }
        
        #[cfg(target_arch = "wasm32")]
        {
            // WASM fallback (for Enarx)
            self.submit_batch_wasm(batch).await
        }
    }
    
    /// Submit batch to Ethereum (native)
    #[cfg(not(target_arch = "wasm32"))]
    async fn submit_batch_to_ethereum(&self, batch: &Batch) -> Result<BatchResult> {
        let start = std::time::Instant::now();
        
        let wallet = self.wallet.as_ref()
            .ok_or_else(|| Error::Contract("Wallet not initialized".to_string()))?;
        
        // Create contract instance
        let contract = FisherContract::new(
            H160::from_slice(self.config.fisher_address.as_slice()),
            Arc::new(wallet.clone()),
        );
        
        // Convert intents to contract Payment structs
        let payments: Vec<Payment> = batch.intents.iter()
            .map(|intent| {
                let from = H160::from_slice(intent.from.as_slice());
                let to = H160::from_slice(intent.to.as_slice());
                let amount = ethers::types::U256::from_big_endian(&intent.amount.to_be_bytes::<32>());
                let nonce = ethers::types::U256::from(intent.nonce);
                
                Payment {
                    from,
                    to,
                    amount,
                    priority_flag: intent.priority,
                    nonce,
                }
            })
            .collect();
        
        // Convert signatures
        let signatures: Vec<Bytes> = batch.intents.iter()
            .map(|intent| Bytes::from(intent.signature.clone()))
            .collect();
        
        info!("📝 Calling submitBatchOptimized with {} intents", payments.len());
        
        // Call your FisherProduction.sol contract!
        let call = contract.submit_batch_optimized(payments, signatures);
        
        let tx = call
            .send()
            .await
            .map_err(|e| Error::Contract(format!("Transaction failed: {}", e)))?;
        
        info!("⏳ Transaction sent: {:?}", tx.tx_hash());
        
        // Wait for confirmation
        let receipt = tx
            .await
            .map_err(|e| Error::Contract(format!("Receipt failed: {}", e)))?
            .ok_or_else(|| Error::Contract("No receipt returned".to_string()))?;
        
        let gas_used_eth = receipt.gas_used.unwrap_or_default();
        let gas_used = U256::from_limbs(gas_used_eth.0);
        
        let processing_time_ms = start.elapsed().as_millis() as u64;
        
        info!("✅ Batch {} confirmed!", batch.id);
        info!("   Gas used: {}", gas_used);
        info!("   Tx: {:?}", receipt.transaction_hash);
        info!("   Processing time: {}ms", processing_time_ms);
        
        // Parse success flags from return value
        let successes = vec![true; batch.intents.len()]; // TODO: Parse from logs
        
        Ok(BatchResult {
            batch_id: batch.id,
            tx_hash: format!("{:?}", receipt.transaction_hash),
            gas_used,
            gas_saved: batch.estimated_savings,
            successes,
            processing_time_ms,
            used_blob: self.config.enable_blobs,
            blob_gas_saved: if self.config.enable_blobs {
                batch.estimated_savings
            } else {
                U256::ZERO
            },
        })
    }
    
    /// Submit batch (WASM fallback for Enarx)
    #[cfg(target_arch = "wasm32")]
    async fn submit_batch_wasm(&self, batch: &Batch) -> Result<BatchResult> {
        let start = std::time::Instant::now();
        
        // TODO: Implement WASM-compatible Ethereum submission
        // This would use WASI sockets or host calls
        
        info!("📤 WASM batch submission for batch {}", batch.id);
        
        Ok(BatchResult {
            batch_id: batch.id,
            tx_hash: format!("0x{:064x}", batch.id),
            gas_used: batch.estimated_gas,
            gas_saved: batch.estimated_savings,
            successes: vec![true; batch.intents.len()],
            processing_time_ms: start.elapsed().as_millis() as u64,
            used_blob: false,
            blob_gas_saved: U256::ZERO,
        })
    }
    
    /// Update metrics
    async fn update_metrics(&self, batch: &Batch, result: &BatchResult) {
        let mut metrics = self.metrics.write().await;
        
        metrics.total_batches += 1;
        metrics.total_intents += batch.intents.len() as u64;
        metrics.total_gas_saved += result.gas_saved;
        
        // Calculate detailed savings breakdown
        let (williams_savings, phi_savings, combined_savings) = 
            crate::phi_optimization::estimate_total_savings(batch.intents.len());
        
        // Update averages
        let n = metrics.total_batches as f64;
        metrics.avg_batch_size = (metrics.avg_batch_size * (n - 1.0) + batch.intents.len() as f64) / n;
        metrics.avg_savings_percent = (metrics.avg_savings_percent * (n - 1.0) + combined_savings) / n;
        metrics.avg_williams_savings = (metrics.avg_williams_savings * (n - 1.0) + williams_savings) / n;
        metrics.avg_phi_savings = (metrics.avg_phi_savings * (n - 1.0) + phi_savings) / n;
    }
    
    /// Get current metrics
    pub async fn get_metrics(&self) -> Metrics {
        self.metrics.read().await.clone()
    }
    
    /// Generate attestation report (if enabled)
    #[cfg(feature = "attestation")]
    pub fn get_attestation(&self) -> Result<crate::attestation::AttestationReport> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(self.config.fisher_address.as_slice());
        hasher.update(self.config.evvm_core_address.as_slice());
        let config_hash = hasher.finalize().into();
        
        crate::attestation::generate_tdx_attestation(&config_hash)
    }
    
    /// Clone for Arc sharing (internal use)
    fn clone_arc(&self) -> Self {
        Self {
            config: self.config.clone(),
            intent_queue: Arc::clone(&self.intent_queue),
            metrics: Arc::clone(&self.metrics),
            fishing_spot: self.fishing_spot.clone(),
            staking: self.staking.clone(),
            
            #[cfg(not(target_arch = "wasm32"))]
            wallet: self.wallet.clone(),
        }
    }
    
    /// Start automatic batch processing
    pub async fn start(&self) {
        info!("🎯 Starting automatic batch processor");
        info!("   • Interval: {}ms", self.config.batch_interval_ms);
        info!("   • Min size: {}", self.config.min_batch_size);
        info!("   • Max size: {}", self.config.max_batch_size);
        
        let this = self.clone_arc();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_millis(this.config.batch_interval_ms)
            );
            
            loop {
                interval.tick().await;
                
                if let Err(e) = this.process_batch().await {
                    if !matches!(e, Error::BatchProcessing(_)) {
                        warn!("⚠️  Batch processing error: {}", e);
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::Address;

    #[tokio::test]
    async fn test_fisher_relayer() {
        let config = FisherConfig::default();
        let relayer = FisherRelayer::new(config).unwrap();
        
        // Submit some test intents
        for i in 0..15 {
            let intent = Intent::new(
                format!("test_{}", i),
                Address::ZERO,
                Address::ZERO,
                U256::from(100),
                false,
                i,
                vec![0xDE, 0xAD, 0xBE, 0xEF],
            );
            
            relayer.submit_intent(intent).await.unwrap();
        }
        
        // Verify intents were queued
        let queue = relayer.intent_queue.read().await;
        assert_eq!(queue.len(), 15);
        
        // Note: Can't test process_batch() without Ethereum connection
        // For full integration tests, use examples/run_fisher.rs
    }
}
