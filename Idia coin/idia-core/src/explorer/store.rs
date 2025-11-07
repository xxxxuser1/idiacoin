//! Block storage implementation

use super::*;
use std::collections::HashMap;

/// Block information (public view)
#[derive(Debug, Clone)]
pub struct BlockInfo {
    /// Block hash
    pub hash: Hash,
    /// Block height
    pub height: u64,
    /// Timestamp
    pub timestamp: u64,
    /// Number of transactions
    pub tx_count: usize,
}

/// Transaction view with privacy protections
#[derive(Debug, Clone)]
pub struct TransactionView {
    /// Transaction hash
    pub hash: Hash,
    /// Block height
    pub height: u64,
    /// Timestamp
    pub timestamp: u64,
    /// Number of inputs
    pub input_count: usize,
    /// Number of outputs
    pub output_count: usize,
    /// Fee (if visible)
    pub fee: Option<u64>,
    /// Outputs visible to view key
    pub visible_outputs: Vec<OutputView>,
}

/// Output view with privacy protections
#[derive(Debug, Clone)]
pub struct OutputView {
    /// Output index
    pub index: u32,
    /// Amount (if visible)
    pub amount: Option<u64>,
    /// One-time public key
    pub stealth_pubkey: String,
}

/// Block storage
pub struct BlockStore {
    /// Blocks by hash
    blocks: HashMap<Hash, Block>,
    /// Block height mapping
    heights: HashMap<u64, Hash>,
    /// Transactions by hash
    transactions: HashMap<Hash, (Hash, usize)>, // (block_hash, tx_index)
}

impl BlockStore {
    /// Create a new block store
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            heights: HashMap::new(),
            transactions: HashMap::new(),
        }
    }

    /// Add a block to storage
    pub fn add_block(&mut self, block: Block) -> Result<(), ExplorerError> {
        let block_hash = block.hash();
        
        // Index transactions
        for (idx, tx) in block.transactions.iter().enumerate() {
            let tx_hash = tx.hash();
            self.transactions.insert(tx_hash, (block_hash, idx));
        }

        // Store block
        self.heights.insert(block.header.height, block_hash);
        self.blocks.insert(block_hash, block);

        Ok(())
    }

    /// Get basic block information
    pub fn get_block_info(&self, hash: &Hash) -> Result<BlockInfo, ExplorerError> {
        let block = self.blocks.get(hash)
            .ok_or(ExplorerError::BlockNotFound)?;

        Ok(BlockInfo {
            hash: *hash,
            height: block.header.height,
            timestamp: block.header.timestamp,
            tx_count: block.transactions.len(),
        })
    }

    /// Get transaction view
    pub fn get_transaction_view(
        &self,
        tx_hash: &Hash,
    ) -> Result<Option<TransactionView>, ExplorerError> {
        let (block_hash, tx_idx) = self.transactions.get(tx_hash)
            .ok_or(ExplorerError::TransactionNotFound)?;

        let block = self.blocks.get(block_hash)
            .ok_or(ExplorerError::BlockNotFound)?;

        let tx = &block.transactions[*tx_idx];

        Ok(Some(TransactionView {
            hash: *tx_hash,
            height: block.header.height,
            timestamp: block.header.timestamp,
            input_count: tx.inputs.len(),
            output_count: tx.outputs.len(),
            fee: Some(tx.fee), // Fee is public
            visible_outputs: vec![], // Only outputs visible to view key
        }))
    }

    /// Get block by height
    pub fn get_block_by_height(&self, height: u64) -> Result<Block, ExplorerError> {
        let hash = self.heights.get(&height)
            .ok_or(ExplorerError::BlockNotFound)?;
        
        self.blocks.get(hash)
            .cloned()
            .ok_or(ExplorerError::BlockNotFound)
    }
}