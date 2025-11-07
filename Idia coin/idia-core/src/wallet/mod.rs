//! Wallet implementation

mod keystore;
mod scanner;
mod transaction_builder;

pub use keystore::*;
pub use scanner::*;
pub use transaction_builder::*;

use crate::crypto::{StealthAddress, KeyImage};
use crate::types::{Transaction, Output, Input, OutputReference};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Wallet error types
#[derive(Debug, thiserror::Error)]
pub enum WalletError {
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Invalid amount")]
    InvalidAmount,
    #[error("Key store error: {0}")]
    KeyStoreError(String),
    #[error("Scanner error: {0}")]
    ScannerError(String),
    #[error("Transaction building error: {0}")]
    TransactionBuildError(String),
}

/// Wallet state
#[derive(Debug)]
pub struct WalletState {
    /// Unspent outputs owned by this wallet
    unspent_outputs: HashMap<OutputReference, Output>,
    /// Key images of spent outputs
    spent_key_images: HashMap<KeyImage, OutputReference>,
    /// Total balance
    balance: u64,
}

/// Wallet configuration
#[derive(Debug, Clone)]
pub struct WalletConfig {
    /// Wallet data directory
    pub data_dir: PathBuf,
    /// Network type (mainnet, testnet)
    pub network: NetworkType,
    /// Default ring size for transactions
    pub ring_size: usize,
}

/// Network type
#[derive(Debug, Clone, Copy, EqualsPartial)]
pub enum NetworkType {
    Mainnet,
    Testnet,
}

/// Main wallet structure
pub struct Wallet {
    /// Wallet configuration
    config: WalletConfig,
    /// Wallet state
    state: Arc<RwLock<WalletState>>,
    /// Key store
    keystore: KeyStore,
    /// Output scanner
    scanner: OutputScanner,
    /// Transaction builder
    tx_builder: TransactionBuilder,
}

impl Wallet {
    /// Create a new wallet
    pub async fn new(config: WalletConfig) -> Result<Self, WalletError> {
        let keystore = KeyStore::new(&config.data_dir)?;
        let scanner = OutputScanner::new();
        let tx_builder = TransactionBuilder::new(config.ring_size);

        let state = Arc::new(RwLock::new(WalletState {
            unspent_outputs: HashMap::new(),
            spent_key_images: HashMap::new(),
            balance: 0,
        }));

        Ok(Self {
            config,
            state,
            keystore,
            scanner,
            tx_builder,
        })
    }

    /// Get the wallet's stealth address
    pub fn get_address(&self) -> Result<StealthAddress, WalletError> {
        self.keystore.get_stealth_address()
    }

    /// Get the current balance
    pub async fn get_balance(&self) -> u64 {
        self.state.read().await.balance
    }

    /// Create a new transaction
    pub async fn create_transaction(
        &self,
        recipient: &StealthAddress,
        amount: u64,
        fee: u64,
    ) -> Result<Transaction, WalletError> {
        let state = self.state.read().await;
        
        // Check if we have enough funds
        if amount + fee > state.balance {
            return Err(WalletError::InsufficientFunds);
        }

        // Build transaction
        self.tx_builder
            .build_transaction(
                &self.keystore,
                &state.unspent_outputs,
                recipient,
                amount,
                fee,
            )
            .map_err(|e| WalletError::TransactionBuildError(e.to_string()))
    }

    /// Process a new block
    pub async fn process_block(&mut self, block: &Block) -> Result<(), WalletError> {
        let mut state = self.state.write().await;
        
        // Scan for our outputs
        for tx in &block.transactions {
            if let Some(new_outputs) = self.scanner.scan_transaction(
                tx,
                &self.keystore.get_stealth_address()?,
            )? {
                // Add new outputs
                for (outref, output) in new_outputs {
                    state.balance += output.amount;
                    state.unspent_outputs.insert(outref, output);
                }
            }

            // Mark spent outputs
            for input in &tx.inputs {
                if let Some(outref) = state.spent_key_images.insert(
                    input.key_image.clone(),
                    input.ring[0].clone(), // Assuming first ring member is real
                ) {
                    if let Some(output) = state.unspent_outputs.remove(&outref) {
                        state.balance -= output.amount;
                    }
                }
            }
        }

        Ok(())
    }
}