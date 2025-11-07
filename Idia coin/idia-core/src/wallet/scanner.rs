//! Output scanner for identifying owned outputs

use super::*;
use crate::crypto::StealthAddress;

/// Scanner for identifying outputs belonging to a wallet
pub struct OutputScanner;

impl OutputScanner {
    /// Create a new output scanner
    pub fn new() -> Self {
        Self
    }

    /// Scan a transaction for outputs belonging to the given stealth address
    pub fn scan_transaction(
        &self,
        tx: &Transaction,
        address: &StealthAddress,
    ) -> Result<Option<HashMap<OutputReference, Output>>, WalletError> {
        let mut owned_outputs = HashMap::new();

        for (idx, output) in tx.outputs.iter().enumerate() {
            // Check if this output is for us
            if address.scan_one_time_key(&output.tx_pubkey, &output.stealth_pubkey) {
                let outref = OutputReference {
                    tx_hash: tx.hash(),
                    output_index: idx as u32,
                };
                owned_outputs.insert(outref, output.clone());
            }
        }

        if owned_outputs.is_empty() {
            Ok(None)
        } else {
            Ok(Some(owned_outputs))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_scanning() {
        let scanner = OutputScanner::new();
        let recipient = StealthAddress::new();
        
        // Create a transaction with an output for our address
        let (output, _) = Output::new(100, &recipient).unwrap();
        let tx = Transaction::new(vec![], vec![output], 1);
        
        // Scan the transaction
        let found = scanner.scan_transaction(&tx, &recipient).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().len(), 1);
        
        // Try scanning with different address
        let other_addr = StealthAddress::new();
        let found = scanner.scan_transaction(&tx, &other_addr).unwrap();
        assert!(found.is_none());
    }
}