//! Transaction builder for creating new transactions

use super::*;
use crate::crypto::{KeyImage, RingSignature, StealthAddress};
use rand::{seq::IteratorRandom, thread_rng};

/// Transaction builder for constructing new transactions
pub struct TransactionBuilder {
    /// Ring size for transactions
    ring_size: usize,
}

impl TransactionBuilder {
    /// Create a new transaction builder
    pub fn new(ring_size: usize) -> Self {
        Self { ring_size }
    }

    /// Build a new transaction
    pub fn build_transaction(
        &self,
        keystore: &KeyStore,
        available_outputs: &HashMap<OutputReference, Output>,
        recipient: &StealthAddress,
        amount: u64,
        fee: u64,
    ) -> Result<Transaction, WalletError> {
        let total_needed = amount + fee;
        
        // Select inputs
        let mut selected_amount = 0u64;
        let mut selected_inputs = Vec::new();
        
        for (outref, output) in available_outputs {
            if selected_amount >= total_needed {
                break;
            }
            
            selected_inputs.push((outref.clone(), output.clone()));
            selected_amount += output.amount;
        }

        if selected_amount < total_needed {
            return Err(WalletError::InsufficientFunds);
        }

        // Create outputs
        let mut outputs = Vec::new();
        
        // Payment output
        let (payment_output, _) = Output::new(amount, recipient)?;
        outputs.push(payment_output);

        // Change output if needed
        if selected_amount > total_needed {
            let change_amount = selected_amount - total_needed;
            let (change_output, _) = Output::new(
                change_amount,
                &keystore.get_stealth_address()?,
            )?;
            outputs.push(change_output);
        }

        // Build ring signatures
        let mut inputs = Vec::new();
        for (outref, output) in selected_inputs {
            // TODO: Select decoy outputs from the blockchain
            let mut ring = vec![outref.clone()];
            
            // Create key image and ring signature
            let key_image = KeyImage(output.stealth_pubkey.compress());
            
            // TODO: Implement proper ring signature creation
            let signature = RingSignature::sign(
                keystore.get_stealth_address()?.derive_private_key(&output.tx_pubkey),
                key_image.clone(),
                &[output.stealth_pubkey],
                0,
            )?;

            inputs.push(Input {
                ring,
                signature,
                key_image,
            });
        }

        Ok(Transaction::new(inputs, outputs, fee))
    }

    /// Select decoy outputs for ring signatures
    fn select_decoys(
        &self,
        real_output: &OutputReference,
        available_decoys: &[OutputReference],
    ) -> Vec<OutputReference> {
        let mut rng = thread_rng();
        let mut ring = vec![real_output.clone()];
        
        // Select random decoys
        ring.extend(
            available_decoys
                .iter()
                .filter(|&x| x != real_output)
                .choose_multiple(&mut rng, self.ring_size - 1)
                .into_iter()
                .cloned(),
        );

        ring
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_transaction_building() {
        let dir = tempdir().unwrap();
        let keystore = KeyStore::new(&dir.path().to_path_buf()).unwrap();
        
        let mut available_outputs = HashMap::new();
        
        // Create some test outputs
        let (output, _) = Output::new(1000, &keystore.get_stealth_address().unwrap()).unwrap();
        let outref = OutputReference {
            tx_hash: [0; 32],
            output_index: 0,
        };
        available_outputs.insert(outref, output);

        let builder = TransactionBuilder::new(11);
        let recipient = StealthAddress::new();
        
        // Try building a transaction
        let tx = builder.build_transaction(
            &keystore,
            &available_outputs,
            &recipient,
            500,
            1,
        ).unwrap();

        assert_eq!(tx.inputs.len(), 1);
        assert_eq!(tx.outputs.len(), 2); // payment + change
        assert_eq!(tx.fee, 1);
    }
}