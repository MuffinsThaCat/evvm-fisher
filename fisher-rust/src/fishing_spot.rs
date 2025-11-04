//! Fishing Spot Client - Collects intents from off-chain sources
//!
//! Implements gasless intent collection from EVVM fishing spots.
//! Users submit intents to fishing spot APIs (zero gas), Fisher collects and batches them.

use crate::{Intent, Result, Error};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for fishing spot connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FishingSpotConfig {
    /// Fishing spot API endpoint
    pub endpoint: String,
    
    /// Poll interval in milliseconds
    pub poll_interval_ms: u64,
    
    /// Maximum intents to fetch per poll
    pub max_batch_size: usize,
    
    /// API authentication token (if required)
    pub auth_token: Option<String>,
}

impl Default for FishingSpotConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://fishing-spot.evvm.io".to_string(),
            poll_interval_ms: 1000,  // Poll every second
            max_batch_size: 1000,
            auth_token: None,
        }
    }
}

/// Fishing spot client for collecting intents
#[derive(Clone)]
pub struct FishingSpotClient {
    config: FishingSpotConfig,
    client: reqwest::Client,
}

/// Response from fishing spot API
#[derive(Debug, Deserialize)]
struct FishingSpotResponse {
    intents: Vec<Intent>,
    total_pending: usize,
    timestamp: u64,
}

impl FishingSpotClient {
    /// Create new fishing spot client
    pub fn new(config: FishingSpotConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { config, client }
    }
    
    /// Poll fishing spot for pending intents
    pub async fn collect_intents(&self) -> Result<Vec<Intent>> {
        let url = format!("{}/api/v1/pending-intents", self.config.endpoint);
        
        let mut request = self.client
            .get(&url)
            .query(&[("limit", self.config.max_batch_size)]);
        
        // Add auth if configured
        if let Some(token) = &self.config.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }
        
        let response = request
            .send()
            .await
            .map_err(|e| Error::Other(format!("Failed to connect to fishing spot: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(Error::Other(format!(
                "Fishing spot returned error: {}",
                response.status()
            )));
        }
        
        let data: FishingSpotResponse = response
            .json()
            .await
            .map_err(|e| Error::Other(format!("Failed to parse response: {}", e)))?;
        
        log::info!(
            "📡 Collected {} intents from fishing spot ({} pending)",
            data.intents.len(),
            data.total_pending
        );
        
        Ok(data.intents)
    }
    
    /// Acknowledge processed intents to fishing spot
    pub async fn acknowledge_intents(&self, intent_ids: &[String]) -> Result<()> {
        let url = format!("{}/api/v1/acknowledge", self.config.endpoint);
        
        let mut request = self.client.post(&url).json(&serde_json::json!({
            "intent_ids": intent_ids
        }));
        
        if let Some(token) = &self.config.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }
        
        let response = request
            .send()
            .await
            .map_err(|e| Error::Other(format!("Failed to acknowledge intents: {}", e)))?;
        
        if !response.status().is_success() {
            log::warn!("Failed to acknowledge intents: {}", response.status());
        }
        
        Ok(())
    }
    
    /// Get fishing spot health and statistics
    pub async fn get_stats(&self) -> Result<FishingSpotStats> {
        let url = format!("{}/api/v1/stats", self.config.endpoint);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::Other(format!("Failed to get stats: {}", e)))?;
        
        let stats = response
            .json()
            .await
            .map_err(|e| Error::Other(format!("Failed to parse stats: {}", e)))?;
        
        Ok(stats)
    }
}

/// Fishing spot statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct FishingSpotStats {
    /// Total intents received
    pub total_received: u64,
    
    /// Pending intents
    pub pending: usize,
    
    /// Processed intents
    pub processed: u64,
    
    /// Average intents per second
    pub intents_per_second: f64,
    
    /// Fishing spot uptime
    pub uptime_seconds: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fishing_spot_config() {
        let config = FishingSpotConfig::default();
        assert_eq!(config.poll_interval_ms, 1000);
        assert_eq!(config.max_batch_size, 1000);
    }
    
    #[tokio::test]
    async fn test_fishing_spot_client_creation() {
        let config = FishingSpotConfig::default();
        let _client = FishingSpotClient::new(config);
        // Just verify it creates successfully
    }
}
