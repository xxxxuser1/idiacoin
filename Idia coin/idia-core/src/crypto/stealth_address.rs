//! Stealth address implementation for one-time addresses

use super::*;
use curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;

/// A stealth address view key pair
#[derive(Debug, Clone)]
pub struct ViewKey {
    pub view_private: Scalar,
    pub view_public: RistrettoPoint,
}

/// A stealth address spend key pair
#[derive(Debug, Clone)]
pub struct SpendKey {
    pub spend_private: Scalar,
    pub spend_public: RistrettoPoint,
}

/// A complete stealth address
#[derive(Debug, Clone)]
pub struct StealthAddress {
    pub view_key: ViewKey,
    pub spend_key: SpendKey,
}

impl StealthAddress {
    /// Generate a new random stealth address
    pub fn new() -> Self {
        let mut rng = OsRng;
        
        // Generate view key
        let view_private = Scalar::random(&mut rng);
        let view_public = RISTRETTO_BASEPOINT_POINT * view_private;
        let view_key = ViewKey { view_private, view_public };
        
        // Generate spend key
        let spend_private = Scalar::random(&mut rng);
        let spend_public = RISTRETTO_BASEPOINT_POINT * spend_private;
        let spend_key = SpendKey { spend_private, spend_public };
        
        Self { view_key, spend_key }
    }

    /// Create a one-time public key for sending to this address
    pub fn generate_one_time_key(&self, r: Scalar) -> (RistrettoPoint, RistrettoPoint) {
        let R = RISTRETTO_BASEPOINT_POINT * r;
        let shared_secret = r * self.view_key.view_public;
        let one_time_pubkey = self.spend_key.spend_public + (shared_secret * RISTRETTO_BASEPOINT_POINT);
        (R, one_time_pubkey)
    }

    /// Check if a one-time public key belongs to this address
    pub fn scan_one_time_key(&self, R: &RistrettoPoint, P: &RistrettoPoint) -> bool {
        let shared_secret = self.view_key.view_private * R;
        let expected = self.spend_key.spend_public + (shared_secret * RISTRETTO_BASEPOINT_POINT);
        P == &expected
    }

    /// Derive the one-time private key for spending
    pub fn derive_private_key(&self, R: &RistrettoPoint) -> Scalar {
        let shared_secret = self.view_key.view_private * R;
        self.spend_key.spend_private + shared_secret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stealth_address() {
        let recipient = StealthAddress::new();
        let mut rng = OsRng;
        let r = Scalar::random(&mut rng);
        
        // Sender creates one-time key
        let (R, P) = recipient.generate_one_time_key(r);
        
        // Recipient scans and identifies the output
        assert!(recipient.scan_one_time_key(&R, &P));
        
        // Recipient can derive private key
        let private_key = recipient.derive_private_key(&R);
        let derived_pubkey = RISTRETTO_BASEPOINT_POINT * private_key;
        assert_eq!(derived_pubkey, P);
    }
}