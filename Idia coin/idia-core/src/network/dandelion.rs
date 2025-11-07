//! Dandelion++ protocol implementation for transaction propagation

use super::*;
use rand::{Rng, thread_rng};
use std::collections::HashMap;
use tokio::time::{Duration, Instant};

/// Dandelion++ phase
#[derive(Debug, Clone, Copy, EqualsPartial)]
pub enum DandelionPhase {
    /// Stem phase (transaction is propagated to a single peer)
    Stem,
    /// Fluff phase (transaction is broadcast to all peers)
    Fluff,
}

/// Dandelion++ transaction state
#[derive(Debug)]
struct DandelionTx {
    /// The transaction
    tx: Transaction,
    /// Current phase
    phase: DandelionPhase,
    /// Timestamp when received
    received_at: Instant,
    /// Next relay peer
    next_peer: Option<PeerId>,
}

/// Dandelion++ protocol handler
pub struct DandelionHandler {
    /// Transactions in stem phase
    stem_txs: HashMap<Hash, DandelionTx>,
    /// Stem graph (outbound peers for stem phase)
    stem_graph: Vec<PeerId>,
    /// Configuration
    config: DandelionConfig,
}

/// Dandelion++ configuration
#[derive(Debug, Clone)]
pub struct DandelionConfig {
    /// Probability of entering fluff phase
    pub fluff_probability: f64,
    /// Maximum time in stem phase
    pub stem_timeout: Duration,
}

impl Default for DandelionConfig {
    fn default() -> Self {
        Self {
            fluff_probability: 0.1,
            stem_timeout: Duration::from_secs(30),
        }
    }
}

impl DandelionHandler {
    /// Create a new Dandelion++ handler
    pub fn new(config: DandelionConfig) -> Self {
        Self {
            stem_txs: HashMap::new(),
            stem_graph: Vec::new(),
            config,
        }
    }

    /// Handle a new transaction
    pub fn handle_transaction(
        &mut self,
        tx: Transaction,
        peers: &[PeerId],
    ) -> Option<(Transaction, Vec<PeerId>)> {
        let tx_hash = tx.hash();
        
        // Check if we've seen this transaction before
        if self.stem_txs.contains_key(&tx_hash) {
            return None;
        }

        // Decide initial phase
        let mut rng = thread_rng();
        let phase = if rng.gen::<f64>() < self.config.fluff_probability {
            DandelionPhase::Fluff
        } else {
            DandelionPhase::Stem
        };

        match phase {
            DandelionPhase::Stem => {
                // Choose next peer in stem phase
                if !self.stem_graph.is_empty() {
                    let next_peer = *self.stem_graph.choose(&mut rng).unwrap();
                    
                    // Store transaction state
                    self.stem_txs.insert(
                        tx_hash,
                        DandelionTx {
                            tx: tx.clone(),
                            phase: DandelionPhase::Stem,
                            received_at: Instant::now(),
                            next_peer: Some(next_peer),
                        },
                    );

                    Some((tx, vec![next_peer]))
                } else {
                    // No stem peers available, fall back to fluff
                    Some((tx, peers.to_vec()))
                }
            }
            DandelionPhase::Fluff => {
                // Broadcast to all peers
                Some((tx, peers.to_vec()))
            }
        }
    }

    /// Process stem transactions that have timed out
    pub fn process_timeouts(&mut self, peers: &[PeerId]) -> Vec<(Transaction, Vec<PeerId>)> {
        let now = Instant::now();
        let mut to_fluff = Vec::new();

        // Find timed out transactions
        self.stem_txs.retain(|_, tx_state| {
            if now.duration_since(tx_state.received_at) > self.config.stem_timeout {
                to_fluff.push((tx_state.tx.clone(), peers.to_vec()));
                false
            } else {
                true
            }
        });

        to_fluff
    }

    /// Update stem graph with new peers
    pub fn update_stem_graph(&mut self, peers: &[PeerId]) {
        let mut rng = thread_rng();
        
        // Randomly select ~10% of peers for stem phase
        self.stem_graph = peers
            .choose_multiple(&mut rng, (peers.len() as f64 * 0.1) as usize)
            .cloned()
            .collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dandelion_phases() {
        let config = DandelionConfig::default();
        let mut handler = DandelionHandler::new(config);

        // Create some test peers
        let peers: Vec<PeerId> = (0..10)
            .map(|_| PeerId::random())
            .collect();
        
        handler.update_stem_graph(&peers);

        // Create a test transaction
        let recipient = crate::crypto::StealthAddress::new();
        let (output, _) = crate::types::Output::new(100, &recipient).unwrap();
        let tx = Transaction::new(vec![], vec![output], 1);

        // Handle transaction multiple times to test both phases
        for _ in 0..100 {
            if let Some((_, relay_peers)) = handler.handle_transaction(tx.clone(), &peers) {
                // Should either relay to one peer (stem) or all peers (fluff)
                assert!(relay_peers.len() == 1 || relay_peers.len() == peers.len());
            }
        }
    }

    #[test]
    fn test_stem_timeout() {
        let mut config = DandelionConfig::default();
        config.stem_timeout = Duration::from_millis(100);
        let mut handler = DandelionHandler::new(config);

        let peers: Vec<PeerId> = (0..10)
            .map(|_| PeerId::random())
            .collect();
        
        handler.update_stem_graph(&peers);

        // Create and add a test transaction
        let recipient = crate::crypto::StealthAddress::new();
        let (output, _) = crate::types::Output::new(100, &recipient).unwrap();
        let tx = Transaction::new(vec![], vec![output], 1);

        // Add to stem phase
        handler.handle_transaction(tx.clone(), &peers);

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(150));

        // Check timeout processing
        let timed_out = handler.process_timeouts(&peers);
        assert!(!timed_out.is_empty());
    }
}