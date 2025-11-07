//! Privacy-preserving blockchain explorer implementation

mod store;
mod views;
mod metrics;

pub use store::*;
pub use views::*;
pub use metrics::*;

use crate::types::{Block, Transaction, Hash};
use crate::crypto::StealthAddress;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Explorer error types
#[derive(Debug, thiserror::Error)]
pub enum ExplorerError {
    #[error("Block not found")]
    BlockNotFound,
    #[error("Transaction not found")]
    TransactionNotFound,
    #[error("Invalid view key")]
    InvalidViewKey,
    #[error("Storage error: {0}")]
    StorageError(String),
}

/// Main explorer structure
pub struct Explorer {
    /// Block storage
    store: Arc<RwLock<BlockStore>>,
    /// View-key authorized views
    views: Arc<RwLock<ViewManager>>,
    /// Privacy-preserving metrics
    metrics: Arc<RwLock<MetricsAggregator>>,
}

impl Explorer {
    /// Create a new explorer instance
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(BlockStore::new())),
            views: Arc::new(RwLock::new(ViewManager::new())),
            metrics: Arc::new(RwLock::new(MetricsAggregator::new())),
        }
    }

    /// Add a new block to the explorer
    pub async fn add_block(&self, block: Block) -> Result<(), ExplorerError> {
        let mut store = self.store.write().await;
        store.add_block(block.clone())?;

        let mut metrics = self.metrics.write().await;
        metrics.process_block(&block);

        Ok(())
    }

    /// Get basic block information (without transaction details)
    pub async fn get_block_info(&self, hash: &Hash) -> Result<BlockInfo, ExplorerError> {
        let store = self.store.read().await;
        store.get_block_info(hash)
    }

    /// Get transaction details if authorized by view key
    pub async fn get_transaction_details(
        &self,
        tx_hash: &Hash,
        view_key: &StealthAddress,
    ) -> Result<Option<TransactionView>, ExplorerError> {
        let store = self.store.read().await;
        let views = self.views.read().await;
        
        if !views.is_authorized(view_key, tx_hash) {
            return Ok(None);
        }

        store.get_transaction_view(tx_hash)
    }

    /// Authorize view key for transaction viewing
    pub async fn authorize_view_key(
        &self,
        view_key: &StealthAddress,
        tx_hash: &Hash,
    ) -> Result<(), ExplorerError> {
        let mut views = self.views.write().await;
        views.authorize(view_key.clone(), *tx_hash);
        Ok(())
    }

    /// Get privacy-preserving metrics
    pub async fn get_metrics(&self) -> NetworkMetrics {
        self.metrics.read().await.get_metrics()
    }
}