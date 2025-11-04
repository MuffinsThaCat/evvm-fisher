//! Fisher Staking Integration
//!
//! Integrates with EVVM's FisherStaking contract for:
//! - Fisher registration and staking
//! - Era-based reward claiming
//! - Staking status monitoring

use crate::{Result, Error};
use alloy_primitives::{Address, U256};
use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
use ethers::{
    prelude::*,
    contract::abigen,
};

// Generate bindings for EVVM FisherStaking contract
#[cfg(not(target_arch = "wasm32"))]
abigen!(
    FisherStakingContract,
    r#"[
        function stakeFisher(uint256 amount) external
        function unstakeFisher(uint256 amount) external
        function claimFisherRewards(uint256 era) external returns (uint256)
        function getStakedAmount(address fisher) external view returns (uint256)
        function getPendingRewards(address fisher, uint256 era) external view returns (uint256)
        function getCurrentEra() external view returns (uint256)
        function isFisherActive(address fisher) external view returns (bool)
        event FisherStaked(address indexed fisher, uint256 amount, uint256 era)
        event FisherUnstaked(address indexed fisher, uint256 amount)
        event RewardsClaimed(address indexed fisher, uint256 era, uint256 amount)
    ]"#
);

/// Fisher staking manager
#[derive(Clone)]
pub struct FisherStaking {
    /// Staking contract address
    pub staking_address: Address,
    
    /// Fisher's own address
    pub fisher_address: Address,
    
    /// Minimum stake required
    pub min_stake: U256,
    
    #[cfg(not(target_arch = "wasm32"))]
    contract: Option<FisherStakingContract<SignerMiddleware<Provider<Http>, LocalWallet>>>,
}

/// Staking status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingStatus {
    /// Whether fisher is registered
    pub is_active: bool,
    
    /// Current staked amount
    pub staked_amount: U256,
    
    /// Current era
    pub current_era: u64,
    
    /// Pending rewards
    pub pending_rewards: U256,
    
    /// Estimated APY
    pub estimated_apy: f64,
}

impl FisherStaking {
    /// Create new staking manager
    pub fn new(
        staking_address: Address,
        fisher_address: Address,
        min_stake: U256,
    ) -> Self {
        Self {
            staking_address,
            fisher_address,
            min_stake,
            #[cfg(not(target_arch = "wasm32"))]
            contract: None,
        }
    }
    
    /// Initialize contract connection
    #[cfg(not(target_arch = "wasm32"))]
    pub fn init_contract(
        &mut self,
        wallet: SignerMiddleware<Provider<Http>, LocalWallet>,
    ) {
        let contract = FisherStakingContract::new(
            H160::from_slice(self.staking_address.as_slice()),
            std::sync::Arc::new(wallet),
        );
        self.contract = Some(contract);
    }
    
    /// Register fisher and stake tokens
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn register_and_stake(&self, amount: U256) -> Result<()> {
        let contract = self.contract.as_ref()
            .ok_or_else(|| Error::Other("Contract not initialized".to_string()))?;
        
        if amount < self.min_stake {
            return Err(Error::Other(format!(
                "Stake amount {} below minimum {}",
                amount, self.min_stake
            )));
        }
        
        log::info!("🎯 Registering fisher with stake: {}", amount);
        
        // Convert to ethers U256
        let amount_eth = ethers::types::U256::from_big_endian(&amount.to_be_bytes::<32>());
        
        let call = contract.stake_fisher(amount_eth);
        let tx = call
            .send()
            .await
            .map_err(|e| Error::Contract(format!("Stake failed: {}", e)))?;
        
        let receipt = tx
            .await
            .map_err(|e| Error::Contract(format!("Transaction failed: {}", e)))?
            .ok_or_else(|| Error::Contract("No receipt".to_string()))?;
        
        log::info!("✅ Fisher staked successfully: {:?}", receipt.transaction_hash);
        
        Ok(())
    }
    
    /// Claim rewards for completed era
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn claim_rewards(&self, era: u64) -> Result<U256> {
        let contract = self.contract.as_ref()
            .ok_or_else(|| Error::Other("Contract not initialized".to_string()))?;
        
        log::info!("💰 Claiming rewards for era {}", era);
        
        let call = contract.claim_fisher_rewards(ethers::types::U256::from(era));
        let tx = call
            .send()
            .await
            .map_err(|e| Error::Contract(format!("Claim failed: {}", e)))?;
        
        let receipt = tx
            .await
            .map_err(|e| Error::Contract(format!("Transaction failed: {}", e)))?
            .ok_or_else(|| Error::Contract("No receipt".to_string()))?;
        
        // Extract reward amount from logs
        let reward = self.extract_reward_from_receipt(&receipt)?;
        
        log::info!("✅ Claimed {} tokens for era {}", reward, era);
        
        Ok(reward)
    }
    
    /// Get current staking status
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn get_status(&self) -> Result<StakingStatus> {
        let contract = self.contract.as_ref()
            .ok_or_else(|| Error::Other("Contract not initialized".to_string()))?;
        
        let fisher_h160 = H160::from_slice(self.fisher_address.as_slice());
        
        // Get staking info
        let is_active = contract.is_fisher_active(fisher_h160).call().await
            .map_err(|e| Error::Contract(format!("Failed to check status: {}", e)))?;
        
        let staked_amount_eth = contract.get_staked_amount(fisher_h160).call().await
            .map_err(|e| Error::Contract(format!("Failed to get stake: {}", e)))?;
        
        let current_era = contract.get_current_era().call().await
            .map_err(|e| Error::Contract(format!("Failed to get era: {}", e)))?
            .as_u64();
        
        let pending_rewards_eth = contract
            .get_pending_rewards(fisher_h160, ethers::types::U256::from(current_era))
            .call()
            .await
            .map_err(|e| Error::Contract(format!("Failed to get rewards: {}", e)))?;
        
        // Convert to alloy U256
        let mut buf = [0u8; 32];
        staked_amount_eth.to_big_endian(&mut buf);
        let staked_amount = U256::from_be_slice(&buf);
        
        pending_rewards_eth.to_big_endian(&mut buf);
        let pending_rewards = U256::from_be_slice(&buf);
        
        Ok(StakingStatus {
            is_active,
            staked_amount,
            current_era,
            pending_rewards,
            estimated_apy: self.calculate_apy(staked_amount, pending_rewards),
        })
    }
    
    /// Calculate estimated APY
    fn calculate_apy(&self, staked: U256, rewards: U256) -> f64 {
        if staked == U256::ZERO {
            return 0.0;
        }
        
        // Simple APY estimate: (rewards / staked) * eras_per_year * 100
        let rewards_f64 = rewards.to_string().parse::<f64>().unwrap_or(0.0);
        let staked_f64 = staked.to_string().parse::<f64>().unwrap_or(1.0);
        
        // Assuming ~365 eras per year (1 per day)
        let apy = (rewards_f64 / staked_f64) * 365.0 * 100.0;
        apy.max(0.0).min(1000.0)  // Cap at reasonable range
    }
    
    /// Extract reward amount from transaction receipt
    #[cfg(not(target_arch = "wasm32"))]
    fn extract_reward_from_receipt(&self, receipt: &TransactionReceipt) -> Result<U256> {
        // Parse RewardsClaimed event from logs
        for log in &receipt.logs {
            // Simple parsing - in production, use proper event decoding
            if log.topics.len() >= 4 {
                let amount_bytes = log.topics[3].as_bytes();
                let amount = U256::from_be_slice(amount_bytes);
                return Ok(amount);
            }
        }
        
        // Default to zero if event not found
        Ok(U256::ZERO)
    }
    
    /// WASM stub
    #[cfg(target_arch = "wasm32")]
    pub async fn register_and_stake(&self, _amount: U256) -> Result<()> {
        Err(Error::Other("Staking not available in WASM".to_string()))
    }
    
    /// WASM stub
    #[cfg(target_arch = "wasm32")]
    pub async fn claim_rewards(&self, _era: u64) -> Result<U256> {
        Err(Error::Other("Staking not available in WASM".to_string()))
    }
    
    /// WASM stub
    #[cfg(target_arch = "wasm32")]
    pub async fn get_status(&self) -> Result<StakingStatus> {
        Err(Error::Other("Staking not available in WASM".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_staking_manager_creation() {
        let staking = FisherStaking::new(
            Address::ZERO,
            Address::ZERO,
            U256::from(1000),
        );
        
        assert_eq!(staking.min_stake, U256::from(1000));
    }
    
    #[test]
    fn test_apy_calculation() {
        let staking = FisherStaking::new(
            Address::ZERO,
            Address::ZERO,
            U256::from(1000),
        );
        
        let staked = U256::from(1000);
        let rewards = U256::from(10);  // 1% per era
        
        let apy = staking.calculate_apy(staked, rewards);
        
        // Should be ~365% APY (1% per day * 365 days)
        assert!(apy >= 300.0 && apy <= 400.0);
    }
}
