//! Example: Run Fisher relayer

use fisher_relayer::{FisherRelayer, FisherConfig, Intent};
use alloy_primitives::{Address, U256};
use std::str::FromStr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Configuration
    let config = FisherConfig {
        rpc_url: "http://localhost:8545".to_string(),
        fisher_address: Address::ZERO,
        evvm_core_address: Address::ZERO,
        min_batch_size: 5,
        max_batch_size: 100,
        batch_interval_ms: 2000,
        enable_attestation: false,
        enable_blobs: true,
        private_key: None,
    };
    
    // Create relayer
    let mut relayer = FisherRelayer::new(config)?;
    
    // Initialize Ethereum connection
    #[cfg(not(target_arch = "wasm32"))]
    relayer.init_ethereum().await?;
    
    println!("🚀 Fisher Relayer started!");
    println!("📊 Submit intents to build batches");
    
    // Start automatic batch processing
    relayer.start().await;
    
    // Example: Submit test intents
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
        
        relayer.submit_intent(intent).await?;
    }
    
    // Keep running
    tokio::signal::ctrl_c().await?;
    
    println!("👋 Shutting down...");
    
    Ok(())
}
