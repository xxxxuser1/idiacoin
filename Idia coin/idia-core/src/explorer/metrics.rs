//! Privacy-preserving network metrics

use super::*;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Network metrics that don't leak privacy
#[derive(Debug, Clone)]
pub struct NetworkMetrics {
    /// Total number of blocks
    pub block_count: u64,
    /// Average block time (last 100 blocks)
    pub avg_block_time: Duration,
    /// Network hashrate estimate (no individual miner data)
    pub estimated_hashrate: u64,
    /// Current difficulty
    pub current_difficulty: u32,
    /// Transaction pool size (count only)
    pub mempool_size: usize,
}

/// Metrics aggregator that preserves privacy
pub struct MetricsAggregator {
    /// Total blocks processed
    block_count: u64,
    /// Recent block timestamps
    recent_blocks: Vec<u64>,
    /// Current difficulty
    current_difficulty: u32,
    /// Mempool size
    mempool_size: usize,
    /// Maximum history to keep
    max_history: usize,
}

impl MetricsAggregator {
    /// Create a new metrics aggregator
    pub fn new() -> Self {
        Self {
            block_count: 0,
            recent_blocks: Vec::new(),
            current_difficulty: 0,
            mempool_size: 0,
            max_history: 100,
        }
    }

    /// Process a new block for metrics
    pub fn process_block(&mut self, block: &Block) {
        self.block_count += 1;
        self.current_difficulty = block.header.difficulty;

        // Update recent blocks
        self.recent_blocks.push(block.header.timestamp);
        if self.recent_blocks.len() > self.max_history {
            self.recent_blocks.remove(0);
        }
    }

    /// Update mempool size
    pub fn update_mempool_size(&mut self, size: usize) {
        self.mempool_size = size;
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> NetworkMetrics {
        let avg_block_time = if self.recent_blocks.len() >= 2 {
            let total_time: u64 = self.recent_blocks
                .windows(2)
                .map(|w| w[1] - w[0])
                .sum();
            Duration::from_secs(total_time / (self.recent_blocks.len() as u64 - 1))
        } else {
            Duration::from_secs(0)
        };

        // Estimate hashrate from difficulty and block time
        let estimated_hashrate = if !avg_block_time.is_zero() {
            (self.current_difficulty as u64) * (2u64.pow(32) / avg_block_time.as_secs())
        } else {
            0
        };

        NetworkMetrics {
            block_count: self.block_count,
            avg_block_time,
            estimated_hashrate,
            current_difficulty: self.current_difficulty,
            mempool_size: self.mempool_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_aggregation() {
        let mut aggregator = MetricsAggregator::new();
        
        // Create some test blocks
        let mut timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        for i in 0..5 {
            let block = Block::new(
                [0; 32],
                i,
                1000,
                vec![],
            );
            aggregator.process_block(&block);
            timestamp += 60; // 1 minute between blocks
        }

        let metrics = aggregator.get_metrics();
        assert_eq!(metrics.block_count, 5);
        assert_eq!(metrics.current_difficulty, 1000);
    }

    #[test]
    fn test_mempool_metrics() {
        let mut aggregator = MetricsAggregator::new();
        
        aggregator.update_mempool_size(42);
        let metrics = aggregator.get_metrics();
        assert_eq!(metrics.mempool_size, 42);
    }
}