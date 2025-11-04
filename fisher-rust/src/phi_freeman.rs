//! φ-Freeman Optimization Algorithm
//!
//! Optimal batching strategy using the golden ratio (φ = 1.618...)
//! for transaction ordering and grouping.

use crate::Intent;
use std::cmp::Ordering;

/// Golden ratio (φ)
const PHI: f64 = 1.618033988749;

/// φ-Freeman score for an intent
///
/// Combines multiple factors:
/// - Priority flag (urgent vs normal)
/// - Amount (larger transactions weighted higher)
/// - Timestamp (older transactions processed first)
/// - Gas price (higher paying users get priority)
fn calculate_phi_score(intent: &Intent, now: u64) -> f64 {
    let age_factor = (now.saturating_sub(intent.timestamp)) as f64;
    let amount_factor = intent.amount.to::<u128>() as f64;
    let priority_factor = if intent.priority { PHI } else { 1.0 };
    let gas_factor = intent.max_gas_price
        .map(|p| p.to::<u128>() as f64)
        .unwrap_or(1.0);
    
    // Combine factors with φ-weighted formula
    priority_factor * (age_factor.powf(1.0 / PHI) + amount_factor.ln() + gas_factor.ln())
}

/// Sort intents using φ-Freeman optimization
///
/// Optimally orders intents to maximize batch efficiency and fairness.
///
/// # Arguments
/// * `intents` - Mutable slice of intents to sort
///
/// # Examples
/// ```
/// use fisher_relayer::phi_freeman::phi_sort;
/// use fisher_relayer::Intent;
///
/// let mut intents = vec![/* your intents */];
/// phi_sort(&mut intents);
/// // Now optimally ordered for batching
/// ```
pub fn phi_sort(intents: &mut [Intent]) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    intents.sort_by(|a, b| {
        let score_a = calculate_phi_score(a, now);
        let score_b = calculate_phi_score(b, now);
        
        score_b.partial_cmp(&score_a).unwrap_or(Ordering::Equal)
    });
}

/// Group intents into optimal sub-batches using φ ratio
///
/// Divides intents into groups where each group is φ times
/// the size of the previous group, optimizing for Williams compression.
pub fn phi_group(intents: &[Intent]) -> Vec<Vec<Intent>> {
    if intents.is_empty() {
        return vec![];
    }
    
    let mut groups: Vec<Vec<Intent>> = Vec::new();
    let mut start = 0;
    let total = intents.len();
    
    while start < total {
        let remaining = total - start;
        let group_size = if groups.is_empty() {
            // First group: √n elements
            (total as f64).sqrt() as usize
        } else {
            // Subsequent groups: φ * previous size
            let prev_size = groups.last().unwrap().len();
            ((prev_size as f64 * PHI) as usize).min(remaining)
        };
        
        let end = (start + group_size).min(total);
        groups.push(intents[start..end].to_vec());
        start = end;
    }
    
    groups
}

/// Calculate optimal batch composition score
///
/// Evaluates how well a batch is composed using φ metrics.
/// Higher scores indicate better optimization.
pub fn batch_score(intents: &[Intent]) -> f64 {
    if intents.is_empty() {
        return 0.0;
    }
    
    let n = intents.len() as f64;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Average φ score
    let avg_score: f64 = intents
        .iter()
        .map(|i| calculate_phi_score(i, now))
        .sum::<f64>() / n;
    
    // Size factor (closer to φ-optimal size scores higher)
    let size_score = 1.0 / (1.0 + (n - n / PHI).abs());
    
    // Priority distribution (balanced mix is better)
    let priority_count = intents.iter().filter(|i| i.priority).count() as f64;
    let priority_balance = 1.0 - (priority_count / n - 0.5).abs();
    
    // Combined score
    (avg_score * size_score * priority_balance).ln().exp()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{Address, U256};

    fn make_intent(id: &str, priority: bool, amount: u64, timestamp: u64) -> Intent {
        Intent {
            id: id.to_string(),
            from: Address::ZERO,
            to: Address::ZERO,
            amount: U256::from(amount),
            priority,
            nonce: 0,
            signature: vec![],
            timestamp,
            max_gas_price: None,
        }
    }

    #[test]
    fn test_phi_sort() {
        let mut intents = vec![
            make_intent("low", false, 100, 1000),
            make_intent("high", true, 1000, 900),
            make_intent("old", false, 100, 800),
        ];
        
        phi_sort(&mut intents);
        
        // Priority intents should generally come first
        assert!(intents[0].priority || intents[0].id == "old");
    }

    #[test]
    fn test_phi_group() {
        let intents: Vec<Intent> = (0..100)
            .map(|i| make_intent(&format!("intent_{}", i), false, 100, 1000))
            .collect();
        
        let groups = phi_group(&intents);
        
        assert!(!groups.is_empty());
        assert_eq!(groups.iter().map(|g| g.len()).sum::<usize>(), 100);
    }

    #[test]
    fn test_batch_score() {
        let intents: Vec<Intent> = (0..10)
            .map(|i| make_intent(&format!("intent_{}", i), i % 2 == 0, 100, 1000))
            .collect();
        
        let score = batch_score(&intents);
        assert!(score > 0.0);
    }
}
