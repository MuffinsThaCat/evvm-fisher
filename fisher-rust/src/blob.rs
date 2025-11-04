//! EIP-4844 Blob Transaction Support
//!
//! Implements blob transactions for massive gas savings on batch data

use crate::{Batch, Result, Error};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Type alias for KZG commitment (48 bytes)
pub type Commitment = Vec<u8>;

/// Type alias for KZG proof (48 bytes)
pub type Proof = Vec<u8>;

/// Maximum blob size (128KB per blob)
pub const BLOB_SIZE: usize = 131_072;

/// Maximum number of blobs per transaction
pub const MAX_BLOBS_PER_TX: usize = 6;

/// Field element size for KZG commitment
pub const FIELD_ELEMENT_SIZE: usize = 32;

/// Number of field elements per blob
pub const FIELD_ELEMENTS_PER_BLOB: usize = 4096;

/// Blob transaction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobTx {
    /// KZG commitment to blob data (48 bytes)
    pub commitment: Commitment,
    
    /// Versioned hash of blob
    pub versioned_hash: [u8; 32],
    
    /// Blob data (off-chain)
    pub blob_data: Vec<u8>,
    
    /// Proof for KZG commitment (48 bytes)
    pub proof: Proof,
}

/// Blob batch encoder
pub struct BlobEncoder;

impl BlobEncoder {
    /// Encode batch into blob format
    pub fn encode_batch(batch: &Batch) -> Result<Vec<BlobTx>> {
        // Serialize batch to bytes
        let batch_bytes = bincode::serialize(batch)
            .map_err(|e| Error::Other(format!("Failed to serialize batch: {}", e)))?;
        
        // Split into blobs if needed (max 128KB per blob)
        let num_blobs = (batch_bytes.len() + BLOB_SIZE - 1) / BLOB_SIZE;
        
        if num_blobs > MAX_BLOBS_PER_TX {
            return Err(Error::BatchTooLarge(format!(
                "Batch requires {} blobs, max is {}",
                num_blobs, MAX_BLOBS_PER_TX
            )));
        }
        
        let mut blobs = Vec::new();
        
        for i in 0..num_blobs {
            let start = i * BLOB_SIZE;
            let end = ((i + 1) * BLOB_SIZE).min(batch_bytes.len());
            let chunk = &batch_bytes[start..end];
            
            let blob_tx = Self::create_blob_tx(chunk)?;
            blobs.push(blob_tx);
        }
        
        Ok(blobs)
    }
    
    /// Create blob transaction from data chunk
    fn create_blob_tx(data: &[u8]) -> Result<BlobTx> {
        // Pad data to blob size
        let mut blob_data = data.to_vec();
        blob_data.resize(BLOB_SIZE, 0);
        
        // Generate KZG commitment (simplified - real impl would use proper KZG)
        let commitment = Self::generate_commitment(&blob_data);
        
        // Generate versioned hash (EIP-4844 format)
        let versioned_hash = Self::generate_versioned_hash(&commitment);
        
        // Generate proof (simplified)
        let proof = Self::generate_proof(&blob_data, &commitment);
        
        Ok(BlobTx {
            commitment,
            versioned_hash,
            blob_data,
            proof,
        })
    }
    
    /// Generate KZG commitment
    fn generate_commitment(data: &[u8]) -> Commitment {
        // Simplified commitment (real impl would use proper KZG)
        let mut hasher = Sha256::new();
        hasher.update(b"COMMITMENT:");
        hasher.update(data);
        let hash = hasher.finalize();
        
        let mut commitment = vec![0u8; 48];
        commitment[..32].copy_from_slice(&hash);
        commitment
    }
    
    /// Generate versioned hash (VERSIONED_HASH_VERSION_KZG + sha256(commitment)[1..])
    fn generate_versioned_hash(commitment: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(commitment);
        let hash = hasher.finalize();
        
        let mut versioned_hash = [0u8; 32];
        versioned_hash[0] = 0x01; // VERSIONED_HASH_VERSION_KZG
        versioned_hash[1..].copy_from_slice(&hash[1..]);
        
        versioned_hash
    }
    
    /// Generate KZG proof
    fn generate_proof(data: &[u8], commitment: &[u8]) -> Proof {
        // Simplified proof (real impl would use proper KZG)
        let mut hasher = Sha256::new();
        hasher.update(b"PROOF:");
        hasher.update(data);
        hasher.update(commitment);
        let hash = hasher.finalize();
        
        let mut proof = vec![0u8; 48];
        proof[..32].copy_from_slice(&hash);
        proof
    }
    
    /// Decode batch from blobs
    pub fn decode_batch(blobs: &[BlobTx]) -> Result<Batch> {
        let mut combined_data = Vec::new();
        
        for blob in blobs {
            combined_data.extend_from_slice(&blob.blob_data);
        }
        
        // Deserialize batch
        let batch: Batch = bincode::deserialize(&combined_data)
            .map_err(|e| Error::Other(format!("Failed to deserialize batch: {}", e)))?;
        
        Ok(batch)
    }
}

/// Calculate gas savings from using blobs vs calldata
pub fn calculate_blob_savings(batch_size_bytes: usize) -> (u64, u64, f64) {
    // Calldata cost: 16 gas per byte
    let calldata_gas = (batch_size_bytes as u64) * 16;
    
    // Blob cost: ~1 gas per byte (varies with network congestion, but much cheaper)
    // Base fee for blob: ~1-3 gas per byte depending on blob gas price
    // We'll use 2 gas as average
    let blob_gas = (batch_size_bytes as u64) * 2;
    
    // Add verification overhead (~100K gas)
    let blob_gas_total = blob_gas + 100_000;
    
    let savings = calldata_gas.saturating_sub(blob_gas_total);
    let savings_pct = (savings as f64 / calldata_gas as f64) * 100.0;
    
    (calldata_gas, blob_gas_total, savings_pct)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Intent, Batch};
    use alloy_primitives::{Address, U256};
    
    #[test]
    fn test_blob_encoding() {
        let batch = create_test_batch(100);
        
        let blobs = BlobEncoder::encode_batch(&batch).unwrap();
        assert!(!blobs.is_empty());
        assert!(blobs.len() <= MAX_BLOBS_PER_TX);
        
        // Verify commitment is generated
        assert!(!blobs[0].commitment.is_empty());
        assert_eq!(blobs[0].commitment.len(), 48);
    }
    
    #[test]
    fn test_blob_roundtrip() {
        let original = create_test_batch(50);
        
        let blobs = BlobEncoder::encode_batch(&original).unwrap();
        let decoded = BlobEncoder::decode_batch(&blobs).unwrap();
        
        assert_eq!(original.intents.len(), decoded.intents.len());
        assert_eq!(original.id, decoded.id);
    }
    
    #[test]
    fn test_blob_savings_calculation() {
        // 1000 intents × 200 bytes = 200KB
        let batch_size = 200_000;
        
        let (calldata, blob, savings_pct) = calculate_blob_savings(batch_size);
        
        println!("Calldata: {} gas", calldata);
        println!("Blob: {} gas", blob);
        println!("Savings: {:.2}%", savings_pct);
        
        // Should save ~80-90%
        assert!(savings_pct > 80.0);
        assert!(savings_pct < 95.0);
    }
    
    #[test]
    fn test_large_batch_multiple_blobs() {
        // Create batch larger than one blob
        let batch = create_test_batch(2000);
        
        let blobs = BlobEncoder::encode_batch(&batch).unwrap();
        
        // Should split into multiple blobs
        assert!(blobs.len() > 1);
        
        // Should decode correctly
        let decoded = BlobEncoder::decode_batch(&blobs).unwrap();
        assert_eq!(batch.intents.len(), decoded.intents.len());
    }
    
    fn create_test_batch(num_intents: usize) -> Batch {
        let intents: Vec<Intent> = (0..num_intents)
            .map(|i| {
                let mut from = [0u8; 20];
                from[0] = (i as u8);
                let mut to = [1u8; 20];
                to[0] = (i as u8);
                
                Intent {
                    id: format!("intent_{}", i),
                    from: Address::from(from),
                    to: Address::from(to),
                    amount: U256::from(1000),
                    priority: false,
                    nonce: i as u64,
                    signature: vec![0u8; 65],
                    timestamp: 1234567890,
                    max_gas_price: Some(U256::from(20_000_000_000u64)),
                }
            })
            .collect();
        
        Batch {
            id: 1,
            intents,
            chunk_size: 100,
            phi_score: 0.5,
            estimated_gas: U256::from(14_000_000),
            estimated_savings: U256::from(226_000_000),
            created_at: 1234567890,
        }
    }
}
