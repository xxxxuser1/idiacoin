//! Transaction structure and implementation

use super::*;
use crate::crypto::{RingSignature, KeyImage};
use std::collections::HashSet;

/// A transaction input, which spends a previous output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    /// Ring of possible input UTXOs
    pub ring: Vec<OutputReference>,
    /// Ring signature proving ownership of one ring member
    pub signature: RingSignature,
    /// Key image to prevent double-spending
    pub key_image: KeyImage,
}

/// A complete transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Version number for future protocol upgrades
    pub version: u8,
    /// Transaction inputs
    pub inputs: Vec<Input>,
    /// Transaction outputs
    pub outputs: Vec<Output>,
    /// Transaction fee (committed to in input/output balance)
    pub fee: u64,
    /// Timestamp
    pub timestamp: u64,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(
        inputs: Vec<Input>,
        outputs: Vec<Output>,
        fee: u64,
    ) -> Self {
        Self {
            version: 1,
            inputs,
            outputs,
            fee,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Get the transaction hash
    pub fn hash(&self) -> Hash {
        hash_of(self)
    }

    /// Verify the entire transaction
    pub fn verify(&self) -> Result<bool, CryptoError> {
        // Verify each output's range proof
        for output in &self.outputs {
            if !output.verify()? {
                return Ok(false);
            }
        }

        // Verify ring signatures
        for input in &self.inputs {
            // TODO: Implement full ring signature verification
            // This requires accessing the UTXO set to get the public keys
        }

        // Verify no duplicate key images
        let mut key_images = HashSet::new();
        for input in &self.inputs {
            if !key_images.insert(input.key_image.0) {
                return Ok(false);
            }
        }

        // TODO: Verify input/output balance using Pedersen commitments
        // sum(input_commitments) = sum(output_commitments) + fee_commitment

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::StealthAddress;

    #[test]
    fn test_transaction_creation() {
        // Create a recipient
        let recipient = StealthAddress::new();
        
        // Create a simple transaction with one output
        let (output, _r) = Output::new(100, &recipient).unwrap();
        let tx = Transaction::new(
            vec![], // No inputs for this test
            vec![output],
            1, // Small fee
        );
        
        assert_eq!(tx.version, 1);
        assert!(tx.timestamp > 0);
        assert!(!tx.hash().iter().all(|&x| x == 0));
    }
}