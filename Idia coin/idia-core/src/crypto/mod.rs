//! Core cryptographic primitives for Idia

mod pedersen;
mod ring_signature;
mod stealth_address;
mod bulletproof;

pub use pedersen::*;
pub use ring_signature::*;
pub use stealth_address::*;
pub use bulletproof::*;

use curve25519_dalek::ristretto::{RistrettoPoint, CompressedRistretto};
use curve25519_dalek::scalar::Scalar;
use rand::rngs::OsRng;
use sha2::{Sha256, Digest};

/// Error types for cryptographic operations
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Invalid key format")]
    InvalidKey,
    #[error("Signature verification failed")]
    SignatureVerification,
    #[error("Range proof verification failed")]
    RangeProofVerification,
    #[error("Invalid amount")]
    InvalidAmount,
    #[error("Invalid commitment")]
    InvalidCommitment,
}