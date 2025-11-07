//! Block structure and implementation

use super::*;
use std::collections::HashSet;

/// A block header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Block version
    pub version: u8,
    /// Hash of the previous block
    pub prev_hash: Hash,
    /// Merkle root of transactions
    pub merkle_root: Hash,
    /// Block timestamp
    pub timestamp: u64,
    /// Block height
    pub height: u64,
    /// Difficulty target
    pub difficulty: u32,
    /// Nonce for proof of work
    pub nonce: u64,
}

/// A complete block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Block header
    pub header: BlockHeader,
    /// Transactions in the block
    pub transactions: Vec<Transaction>,
}

impl Block {
    /// Create a new block
    pub fn new(
        prev_hash: Hash,
        height: u64,
        difficulty: u32,
        transactions: Vec<Transaction>,
    ) -> Self {
        let merkle_root = Self::calculate_merkle_root(&transactions);
        
        let header = BlockHeader {
            version: 1,
            prev_hash,
            merkle_root,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            height,
            difficulty,
            nonce: 0,
        };

        Self {
            header,
            transactions,
        }
    }

    /// Calculate the merkle root of the transactions
    fn calculate_merkle_root(transactions: &[Transaction]) -> Hash {
        if transactions.is_empty() {
            return [0; 32];
        }

        // Get transaction hashes
        let mut hashes: Vec<Hash> = transactions.iter()
            .map(|tx| tx.hash())
            .collect();

        // Build merkle tree
        while hashes.len() > 1 {
            if hashes.len() % 2 != 0 {
                hashes.push(hashes.last().unwrap().clone());
            }

            let mut new_hashes = Vec::with_capacity(hashes.len() / 2);
            for chunk in hashes.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(&chunk[0]);
                hasher.update(&chunk[1]);
                new_hashes.push(hasher.finalize().into());
            }
            hashes = new_hashes;
        }

        hashes[0]
    }

    /// Get the block hash
    pub fn hash(&self) -> Hash {
        hash_of(&self.header)
    }

    /// Verify the entire block
    pub fn verify(&self) -> Result<bool, CryptoError> {
        // Verify merkle root
        if self.header.merkle_root != Self::calculate_merkle_root(&self.transactions) {
            return Ok(false);
        }

        // Verify each transaction
        for tx in &self.transactions {
            if !tx.verify()? {
                return Ok(false);
            }
        }

        // Verify proof of work
        // TODO: Implement proper PoW verification
        
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_creation() {
        let prev_hash = [0; 32];
        let height = 1;
        let difficulty = 1;
        
        let block = Block::new(
            prev_hash,
            height,
            difficulty,
            vec![], // Empty block for this test
        );
        
        assert_eq!(block.header.version, 1);
        assert_eq!(block.header.height, height);
        assert_eq!(block.header.prev_hash, prev_hash);
        assert!(block.header.timestamp > 0);
    }

    #[test]
    fn test_merkle_root() {
        let recipient = crate::crypto::StealthAddress::new();
        let (output, _) = Output::new(100, &recipient).unwrap();
        
        let tx = Transaction::new(vec![], vec![output], 1);
        let block = Block::new([0; 32], 1, 1, vec![tx]);
        
        assert!(!block.header.merkle_root.iter().all(|&x| x == 0));
        assert_eq!(
            block.header.merkle_root,
            Block::calculate_merkle_root(&block.transactions)
        );
    }
}