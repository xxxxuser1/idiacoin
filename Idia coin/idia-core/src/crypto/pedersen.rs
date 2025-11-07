//! Pedersen commitment implementation for confidential transactions

use super::*;
use merlin::Transcript;

/// A Pedersen commitment of the form `value * G + blinding * H`
#[derive(Debug, Clone)]
pub struct PedersenCommitment(pub CompressedRistretto);

impl PedersenCommitment {
    /// Create a new Pedersen commitment to the given value with a random blinding factor
    pub fn new(value: u64) -> (Self, Scalar) {
        let mut rng = OsRng;
        let blinding = Scalar::random(&mut rng);
        let commitment = Self::with_blinding(value, blinding);
        (commitment, blinding)
    }

    /// Create a commitment with a specific blinding factor
    pub fn with_blinding(value: u64, blinding: Scalar) -> Self {
        let value_scalar = Scalar::from(value);
        let point = RISTRETTO_BASEPOINT_TABLE * value_scalar + RISTRETTO_H_TABLE * blinding;
        Self(point.compress())
    }

    /// Verify that a commitment opens to a specific value with a given blinding factor
    pub fn verify(&self, value: u64, blinding: Scalar) -> bool {
        let check = Self::with_blinding(value, blinding);
        self.0 == check.0
    }

    /// Add two commitments together
    pub fn add(&self, other: &Self) -> Result<Self, CryptoError> {
        let p1 = self.0.decompress().ok_or(CryptoError::InvalidCommitment)?;
        let p2 = other.0.decompress().ok_or(CryptoError::InvalidCommitment)?;
        Ok(Self((p1 + p2).compress()))
    }
}

// Constants for commitment calculation
lazy_static! {
    static ref RISTRETTO_BASEPOINT_TABLE: RistrettoBasepointTable = RistrettoBasepointTable::create(&RISTRETTO_BASEPOINT_POINT);
    static ref RISTRETTO_H_TABLE: RistrettoBasepointTable = {
        let h = RistrettoPoint::hash_from_bytes::<Sha256>(b"Idia_H");
        RistrettoBasepointTable::create(&h)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pedersen_commitment() {
        let value = 42u64;
        let (comm, blinding) = PedersenCommitment::new(value);
        assert!(comm.verify(value, blinding));
        assert!(!comm.verify(value + 1, blinding));
    }

    #[test]
    fn test_commitment_homomorphism() {
        let (c1, b1) = PedersenCommitment::new(40);
        let (c2, b2) = PedersenCommitment::new(2);
        let sum = c1.add(&c2).unwrap();
        
        // Check that the sum commitment opens to the sum of values
        let sum_blinding = b1 + b2;
        assert!(sum.verify(42, sum_blinding));
    }
}