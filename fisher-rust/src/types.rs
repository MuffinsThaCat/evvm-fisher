//! Core types for Fisher relayer

use serde::{Deserialize, Serialize};
use alloy_primitives::{Address, U256};
use std::time::{SystemTime, UNIX_EPOCH};

/// User intent submitted to Fisher
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    /// Unique intent ID
    pub id: String,
    
    /// Sender address
    pub from: Address,
    
    /// Recipient address
    pub to: Address,
    
    /// Amount to transfer
    pub amount: U256,
    
    /// Priority flag for urgent transactions
    pub priority: bool,
    
    /// User nonce
    pub nonce: u64,
    
    /// EIP-191 signature
    pub signature: Vec<u8>,
    
    /// Timestamp (Unix epoch)
    pub timestamp: u64,
    
    /// Gas price user is willing to pay
    pub max_gas_price: Option<U256>,
}

impl Intent {
    /// Create new intent with current timestamp
    pub fn new(
        id: String,
        from: Address,
        to: Address,
        amount: U256,
        priority: bool,
        nonce: u64,
        signature: Vec<u8>,
    ) -> Self {
        Self {
            id,
            from,
            to,
            amount,
            priority,
            nonce,
            signature,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            max_gas_price: None,
        }
    }

    /// Calculate intent hash for ordering
    pub fn hash(&self) -> [u8; 32] {
        use sha3::{Digest, Keccak256};
        let mut hasher = Keccak256::new();
        hasher.update(self.id.as_bytes());
        hasher.update(self.from.as_slice());
        hasher.update(self.to.as_slice());
        hasher.update(&self.amount.to_be_bytes::<32>());
        hasher.update(&self.nonce.to_le_bytes());
        hasher.finalize().into()
    }

    /// Verify EIP-191 signature
    pub fn verify_signature(&self) -> bool {
        // TODO: Implement EIP-191 verification
        !self.signature.is_empty()
    }
}

/// Optimized batch of intents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Batch {
    /// Batch ID
    pub id: u64,
    
    /// Intents in this batch
    pub intents: Vec<Intent>,
    
    /// Williams chunk size used
    pub chunk_size: usize,
    
    /// φ-Freeman optimization score
    pub phi_score: f64,
    
    /// Estimated gas usage
    pub estimated_gas: U256,
    
    /// Estimated gas savings
    pub estimated_savings: U256,
    
    /// Creation timestamp
    pub created_at: u64,
}

impl Batch {
    /// Calculate savings percentage
    pub fn savings_percent(&self) -> f64 {
        if self.estimated_gas.is_zero() {
            return 0.0;
        }
        
        let total = self.estimated_gas + self.estimated_savings;
        (self.estimated_savings.to::<u128>() as f64 / total.to::<u128>() as f64) * 100.0
    }
}

/// Batch processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    /// Batch ID
    pub batch_id: u64,
    
    /// Transaction hash
    pub tx_hash: String,
    
    /// Actual gas used
    pub gas_used: U256,
    
    /// Gas saved
    pub gas_saved: U256,
    
    /// Success flags for each intent
    pub successes: Vec<bool>,
    
    /// Processing time (milliseconds)
    pub processing_time_ms: u64,
    
    /// Whether blob transaction was used
    pub used_blob: bool,
    
    /// Blob gas savings (if applicable)
    pub blob_gas_saved: U256,
}

/// Fisher configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FisherConfig {
    /// Ethereum RPC URL
    pub rpc_url: String,
    
    /// Fisher contract address
    pub fisher_address: Address,
    
    /// EVVM Core contract address
    pub evvm_core_address: Address,
    
    /// Minimum batch size
    pub min_batch_size: usize,
    
    /// Maximum batch size
    pub max_batch_size: usize,
    
    /// Batch interval (milliseconds)
    pub batch_interval_ms: u64,
    
    /// Enable TDX attestation
    pub enable_attestation: bool,
    
    /// Enable EIP-4844 blob transactions
    pub enable_blobs: bool,
    
    /// Relayer private key (encrypted in TEE)
    #[serde(skip_serializing)]
    pub private_key: Option<String>,
}

impl FisherConfig {
    /// Load configuration from JSON file
    pub async fn load(path: impl AsRef<std::path::Path>) -> crate::Result<Self> {
        let contents = tokio::fs::read_to_string(path).await
            .map_err(|e| crate::Error::Config(format!("Failed to read config: {}", e)))?;
        
        let config: Self = serde_json::from_str(&contents)
            .map_err(|e| crate::Error::Config(format!("Failed to parse config: {}", e)))?;
        
        Ok(config)
    }
}

impl Default for FisherConfig {
    fn default() -> Self {
        Self {
            rpc_url: "http://localhost:8545".to_string(),
            fisher_address: Address::ZERO,
            evvm_core_address: Address::ZERO,
            min_batch_size: 10,
            max_batch_size: 1000,
            batch_interval_ms: 5000,
            enable_attestation: true,
            enable_blobs: true,  // Enable blobs by default for best savings
            private_key: None,
        }
    }
}

/// Metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    /// Total batches processed
    pub total_batches: u64,
    
    /// Total intents processed
    pub total_intents: u64,
    
    /// Total gas saved
    pub total_gas_saved: U256,
    
    /// Average batch size
    pub avg_batch_size: f64,
    
    /// Average savings percent (combined Williams + φ-optimization)
    pub avg_savings_percent: f64,
    
    /// Average Williams compression savings
    pub avg_williams_savings: f64,
    
    /// Average φ-optimization savings
    pub avg_phi_savings: f64,
    
    /// EIP-4844 blob savings
    pub avg_blob_savings: f64,
    
    /// Total batches using blobs
    pub blob_batches: u64,
    
    /// Average processing time (ms)
    pub avg_processing_time_ms: f64,
}

impl Metrics {
    /// Display human-readable summary
    pub fn summary(&self) -> String {
        let blob_info = if self.blob_batches > 0 {
            format!(
                "\n             EIP-4844 blobs:        {:.2}% ({} batches)",
                self.avg_blob_savings,
                self.blob_batches
            )
        } else {
            String::new()
        };
        
        format!(
            "📊 Fisher Metrics Summary\n\
             ═══════════════════════════════════════\n\
             Batches processed:     {}\n\
             Intents processed:     {}\n\
             Total gas saved:       {} gas\n\
             \n\
             💰 Gas Savings Breakdown:\n\
             Williams compression:  {:.2}%\n\
             φ-optimization:        {:.2}%{}\n\
             Combined total:        {:.2}%\n\
             \n\
             📈 Performance:\n\
             Avg batch size:        {:.1} intents\n\
             Avg processing time:   {:.1}ms\n\
             ═══════════════════════════════════════",
            self.total_batches,
            self.total_intents,
            self.total_gas_saved,
            self.avg_williams_savings,
            self.avg_phi_savings,
            blob_info,
            self.avg_savings_percent,
            self.avg_batch_size,
            self.avg_processing_time_ms,
        )
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            total_batches: 0,
            total_intents: 0,
            total_gas_saved: U256::ZERO,
            avg_batch_size: 0.0,
            avg_savings_percent: 0.0,
            avg_williams_savings: 0.0,
            avg_phi_savings: 0.0,
            avg_blob_savings: 0.0,
            blob_batches: 0,
            avg_processing_time_ms: 0.0,
        }
    }
}
