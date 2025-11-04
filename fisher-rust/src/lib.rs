//! # Fisher Relayer - Production-Grade Intent Batcher
//!
//! High-performance intent batching system with Williams compression (O(√n log n))
//! and φ-Freeman optimization, designed to run in Intel TDX trusted execution environments.
//!
//! ## Features
//! - 91-95% gas savings through Williams compression + φ-optimization
//! - MEV protection via TDX confidential computing
//! - φ-Freeman optimal batching algorithm
//! - Era-based fee tracking for deterministic computation
//! - Hardware-backed attestation
//! - Sub-second batch processing
//!
//! ## Architecture
//! ```text
//! Users → EVVM Fishing Spots → Fisher (TDX) → Ethereum
//!         (gasless submit)      (batch+optimize)  (settle)
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs, rust_2018_idioms)]

pub mod types;
pub mod relayer;
pub mod williams;
pub mod phi_freeman;
pub mod phi_optimization;

pub mod attestation;
pub mod metrics;
pub mod error;
pub mod blob;
pub mod fishing_spot;
pub mod staking;

// Re-export main types
pub use types::*;
pub use relayer::FisherRelayer;
pub use error::{Error, Result};
pub use blob::{BlobEncoder, BlobTx, calculate_blob_savings};
pub use fishing_spot::{FishingSpotClient, FishingSpotConfig, FishingSpotStats};
pub use staking::{FisherStaking, StakingStatus};

/// Fisher version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Maximum batch size for safety
pub const MAX_BATCH_SIZE: usize = 10_000;

/// Minimum batch size for efficiency
pub const MIN_BATCH_SIZE: usize = 10;
