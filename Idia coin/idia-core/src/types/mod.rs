//! Core types for the Idia blockchain

mod block;
mod transaction;
mod utxo;

pub use block::*;
pub use transaction::*;
pub use utxo::*;

use std::time::SystemTime;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

/// Hash type used throughout the system
pub type Hash = [u8; 32];

/// Compute SHA-256 hash of serialized data
pub fn hash_of<T: Serialize>(data: &T) -> Hash {
    let serialized = bincode::serialize(data).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(serialized);
    hasher.finalize().into()
}