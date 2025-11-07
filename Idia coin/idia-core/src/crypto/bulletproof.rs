//! Bulletproofs range proof implementation

use super::*;
use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
use merlin::Transcript;

/// A wrapper for Bulletproofs range proof
#[derive(Debug, Clone)]
pub struct RangeProofWrapper {
    proof: RangeProof,
    value: u64,
    blinding: Scalar,
}

impl RangeProofWrapper {
    /// Create a new range proof for a value
    pub fn new(value: u64) -> Result<(Self, PedersenCommitment), CryptoError> {
        let mut rng = OsRng;
        let blinding = Scalar::random(&mut rng);

        // Generate Pedersen commitment
        let commitment = PedersenCommitment::with_blinding(value, blinding);
        
        // Setup bulletproofs generators
        let pc_gens = PedersenGens::default();
        let bp_gens = BulletproofGens::new(64, 1);

        // Create the proof
        let mut transcript = Transcript::new(b"idia-range-proof");
        let (proof, _) = RangeProof::prove_single(
            &bp_gens,
            &pc_gens,
            &mut transcript,
            value,
            &blinding,
            32,  // bits in range
        ).map_err(|_| CryptoError::RangeProofVerification)?;

        Ok((Self { proof, value, blinding }, commitment))
    }

    /// Verify a range proof
    pub fn verify(&self, commitment: &PedersenCommitment) -> Result<bool, CryptoError> {
        let pc_gens = PedersenGens::default();
        let bp_gens = BulletproofGens::new(64, 1);
        
        let mut transcript = Transcript::new(b"idia-range-proof");
        
        self.proof
            .verify_single(
                &bp_gens,
                &pc_gens,
                &mut transcript,
                &commitment.0.decompress().ok_or(CryptoError::InvalidCommitment)?,
                32,  // bits in range
            )
            .map_err(|_| CryptoError::RangeProofVerification)?;
            
        Ok(true)
    }

    /// Get the value and blinding factor
    pub fn get_value_blinding(&self) -> (u64, Scalar) {
        (self.value, self.blinding)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_proof() {
        let value = 42u64;
        let (proof, commitment) = RangeProofWrapper::new(value).unwrap();
        
        // Verify the proof
        assert!(proof.verify(&commitment).unwrap());
        
        // Check that the commitment opens correctly
        let (proven_value, blinding) = proof.get_value_blinding();
        assert_eq!(value, proven_value);
        assert!(commitment.verify(value, blinding));
    }

    #[test]
    fn test_range_proof_out_of_range() {
        let value = u64::MAX;  // This should be too large for 32-bit range proof
        assert!(RangeProofWrapper::new(value).is_err());
    }
}