//! UTXO (Unspent Transaction Output) implementation

use super::*;
use crate::crypto::{PedersenCommitment, RangeProofWrapper, StealthAddress};
use curve25519_dalek::ristretto::RistrettoPoint;

/// A transaction output, which includes the commitment and range proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    /// Pedersen commitment to the amount
    pub commitment: PedersenCommitment,
    /// Range proof showing amount is valid
    pub range_proof: RangeProofWrapper,
    /// One-time public key (stealth address)
    pub stealth_pubkey: RistrettoPoint,
    /// Transaction public key (R)
    pub tx_pubkey: RistrettoPoint,
}

/// Reference to a previous output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputReference {
    /// Hash of the transaction containing the output
    pub tx_hash: Hash,
    /// Index of the output in the transaction
    pub output_index: u32,
}

impl Output {
    /// Create a new output with the given amount and recipient's stealth address
    pub fn new(
        amount: u64,
        recipient: &StealthAddress,
    ) -> Result<(Self, Scalar), CryptoError> {
        // Create commitment and range proof
        let (range_proof, commitment) = RangeProofWrapper::new(amount)?;
        
        // Generate one-time keys for the recipient
        let mut rng = OsRng;
        let r = Scalar::random(&mut rng);
        let (tx_pubkey, stealth_pubkey) = recipient.generate_one_time_key(r);
        
        Ok((Self {
            commitment,
            range_proof,
            stealth_pubkey,
            tx_pubkey,
        }, r))
    }

    /// Verify that this output is valid (range proof verifies)
    pub fn verify(&self) -> Result<bool, CryptoError> {
        self.range_proof.verify(&self.commitment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_creation_and_verification() {
        let recipient = StealthAddress::new();
        let amount = 100u64;
        
        let (output, _r) = Output::new(amount, &recipient).unwrap();
        assert!(output.verify().unwrap());
    }
}