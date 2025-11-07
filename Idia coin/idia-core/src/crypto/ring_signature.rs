//! Ring signature implementation (MLSAG - Multilayered Linkable Spontaneous Anonymous Group)

use super::*;
use merlin::Transcript;

/// A key image for preventing double-spending
#[derive(Debug, Clone)]
pub struct KeyImage(pub CompressedRistretto);

/// A ring signature
#[derive(Debug, Clone)]
pub struct RingSignature {
    pub c: Vec<Scalar>,
    pub r: Vec<Vec<Scalar>>,
    pub key_image: KeyImage,
}

impl RingSignature {
    /// Create a new ring signature
    /// * `secret_key` - The real input's private key
    /// * `key_image` - The key image of the real input
    /// * `public_keys` - The ring of public keys (including the real one)
    /// * `real_index` - The position of the real key in the ring
    pub fn sign(
        secret_key: Scalar,
        key_image: KeyImage,
        public_keys: &[RistrettoPoint],
        real_index: usize,
    ) -> Result<Self, CryptoError> {
        if real_index >= public_keys.len() {
            return Err(CryptoError::InvalidKey);
        }

        let n = public_keys.len();
        let mut rng = OsRng;
        
        // Generate random scalars for the real input
        let alpha = Scalar::random(&mut rng);
        
        // Initialize vectors for signature components
        let mut c = vec![Scalar::zero(); n];
        let mut r = vec![vec![Scalar::zero(); 1]; n];
        
        // Create a transcript for Fiat-Shamir
        let mut transcript = Transcript::new(b"idia-ring-signature");
        
        // Initial commitment
        let L = RISTRETTO_BASEPOINT_POINT * alpha;
        transcript.append_message(b"L", L.compress().as_bytes());
        
        // Generate challenge
        let mut challenge_bytes = [0u8; 32];
        transcript.challenge_bytes(b"c", &mut challenge_bytes);
        c[(real_index + 1) % n] = Scalar::from_bytes_mod_order(challenge_bytes);
        
        // Complete the ring
        for i in 1..n {
            let idx = (real_index + i) % n;
            let random = Scalar::random(&mut rng);
            r[idx][0] = random;
            
            let point = RISTRETTO_BASEPOINT_POINT * random + public_keys[idx] * c[idx];
            transcript.append_message(b"point", point.compress().as_bytes());
            
            if idx != real_index {
                transcript.challenge_bytes(b"c", &mut challenge_bytes);
                c[(idx + 1) % n] = Scalar::from_bytes_mod_order(challenge_bytes);
            }
        }
        
        // Close the ring
        r[real_index][0] = alpha - c[real_index] * secret_key;
        
        Ok(Self {
            c,
            r,
            key_image,
        })
    }

    /// Verify a ring signature
    pub fn verify(&self, public_keys: &[RistrettoPoint]) -> Result<bool, CryptoError> {
        if public_keys.len() != self.c.len() || public_keys.len() != self.r.len() {
            return Err(CryptoError::SignatureVerification);
        }

        let mut transcript = Transcript::new(b"idia-ring-signature");
        
        // Verify the ring
        for i in 0..public_keys.len() {
            let point = RISTRETTO_BASEPOINT_POINT * self.r[i][0] + 
                       public_keys[i] * self.c[i];
            transcript.append_message(b"point", point.compress().as_bytes());
            
            let mut challenge_bytes = [0u8; 32];
            transcript.challenge_bytes(b"c", &mut challenge_bytes);
            let expected_c = Scalar::from_bytes_mod_order(challenge_bytes);
            
            if expected_c != self.c[(i + 1) % public_keys.len()] {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_signature() {
        let mut rng = OsRng;
        
        // Generate some keypairs for the ring
        let mut public_keys = Vec::new();
        let mut secret_keys = Vec::new();
        
        for _ in 0..5 {
            let secret = Scalar::random(&mut rng);
            let public = RISTRETTO_BASEPOINT_POINT * secret;
            secret_keys.push(secret);
            public_keys.push(public);
        }
        
        // Create a key image for our real input
        let real_idx = 2;
        let key_image = KeyImage((RISTRETTO_BASEPOINT_POINT * secret_keys[real_idx]).compress());
        
        // Create and verify a ring signature
        let sig = RingSignature::sign(
            secret_keys[real_idx],
            key_image.clone(),
            &public_keys,
            real_idx,
        ).unwrap();
        
        assert!(sig.verify(&public_keys).unwrap());
    }
}