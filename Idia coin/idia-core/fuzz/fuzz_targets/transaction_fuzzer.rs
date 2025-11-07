#![no_main]
use libfuzzer_sys::fuzz_target;
use idia_core::types::{Transaction, Block, hash_of};
use idia_core::crypto::{StealthAddress, PedersenCommitment};

fuzz_target!(|data: &[u8]| {
    if data.len() < 8 {
        return;
    }

    // Try to create and verify transactions with fuzzed data
    let amount = u64::from_le_bytes(data[0..8].try_into().unwrap());
    if amount == 0 {
        return;
    }

    // Create transaction components
    let recipient = StealthAddress::new();
    if let Ok((output, _)) = crate::types::Output::new(amount, &recipient) {
        // Verify output
        let _ = output.verify();

        // Create and verify commitment
        let (commitment, blinding) = PedersenCommitment::new(amount);
        let _ = commitment.verify(amount, blinding);
    }
});