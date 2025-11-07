use std::collections::HashMap;
use chrono::{DateTime, Utc};

pub struct LiquidityPool {
    pub total_liquidity: u64,
    pub providers: HashMap<String, LiquidityProvider>,
    pub pairs: HashMap<String, TradingPair>,
    pub incentives: LiquidityIncentives,
}

pub struct LiquidityProvider {
    pub address: String,
    pub liquidity_provided: u64,
    pub rewards_earned: u64,
    pub last_deposit: DateTime<Utc>,
    pub lock_period: Option<u64>,
}

pub struct TradingPair {
    pub base_token: String,
    pub quote_token: String,
    pub base_reserve: u64,
    pub quote_reserve: u64,
    pub last_price: f64,
    pub volume_24h: u64,
}

pub struct LiquidityIncentives {
    pub reward_rate: f64,
    pub bonus_multipliers: Vec<(u64, f64)>, // (days, multiplier)
    pub total_rewards_distributed: u64,
}

impl LiquidityPool {
    pub fn new() -> Self {
        Self {
            total_liquidity: 0,
            providers: HashMap::new(),
            pairs: HashMap::new(),
            incentives: LiquidityIncentives {
                reward_rate: 0.1, // 10% APY base rate
                bonus_multipliers: vec![
                    (30, 1.2),   // 20% bonus for 30-day lock
                    (90, 1.5),   // 50% bonus for 90-day lock
                    (180, 2.0),  // 100% bonus for 180-day lock
                ],
                total_rewards_distributed: 0,
            },
        }
    }

    pub fn add_liquidity(
        &mut self,
        provider: String,
        amount: u64,
        lock_period: Option<u64>,
    ) -> Result<(), LiquidityError> {
        let provider_info = LiquidityProvider {
            address: provider.clone(),
            liquidity_provided: amount,
            rewards_earned: 0,
            last_deposit: Utc::now(),
            lock_period,
        };

        self.providers.insert(provider, provider_info);
        self.total_liquidity += amount;

        Ok(())
    }

    pub fn calculate_rewards(&self, provider: &LiquidityProvider) -> u64 {
        let base_reward = (provider.liquidity_provided as f64 * self.incentives.reward_rate) as u64;
        
        if let Some(lock_days) = provider.lock_period {
            let multiplier = self.get_bonus_multiplier(lock_days);
            (base_reward as f64 * multiplier) as u64
        } else {
            base_reward
        }
    }

    fn get_bonus_multiplier(&self, lock_days: u64) -> f64 {
        self.incentives.bonus_multipliers
            .iter()
            .filter(|(days, _)| *days <= lock_days)
            .map(|(_, multiplier)| *multiplier)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(1.0)
    }
}

pub struct MarketMaker {
    pub pools: HashMap<String, LiquidityPool>,
    pub order_book: OrderBook,
    pub amm_params: AMMParameters,
}

pub struct OrderBook {
    pub bids: Vec<Order>,
    pub asks: Vec<Order>,
    pub last_price: f64,
}

pub struct Order {
    pub id: String,
    pub price: f64,
    pub amount: u64,
    pub side: OrderSide,
    pub timestamp: DateTime<Utc>,
}

pub enum OrderSide {
    Buy,
    Sell,
}

pub struct AMMParameters {
    pub fee_tier: u64,
    pub slippage_tolerance: f64,
    pub price_impact_limit: f64,
}

impl MarketMaker {
    pub fn new() -> Self {
        Self {
            pools: HashMap::new(),
            order_book: OrderBook {
                bids: Vec::new(),
                asks: Vec::new(),
                last_price: 0.0,
            },
            amm_params: AMMParameters {
                fee_tier: 30,              // 0.3% fee
                slippage_tolerance: 0.01,   // 1% slippage tolerance
                price_impact_limit: 0.05,   // 5% price impact limit
            },
        }
    }

    pub fn calculate_swap_amount(
        &self,
        input_amount: u64,
        input_reserve: u64,
        output_reserve: u64,
    ) -> u64 {
        // Using constant product formula: x * y = k
        let input_with_fee = input_amount * (10000 - self.amm_params.fee_tier);
        let numerator = input_with_fee * output_reserve;
        let denominator = (input_reserve * 10000) + input_with_fee;
        
        numerator / denominator
    }

    pub fn get_price_impact(&self, input_amount: u64, reserve: u64) -> f64 {
        (input_amount as f64 / reserve as f64) * 100.0
    }

    pub fn execute_swap(
        &mut self,
        input_token: String,
        output_token: String,
        amount: u64,
    ) -> Result<SwapResult, SwapError> {
        let pool = self.pools.get_mut(&format!("{}-{}", input_token, output_token))
            .ok_or(SwapError::PoolNotFound)?;

        let pair = pool.pairs.get_mut(&format!("{}-{}", input_token, output_token))
            .ok_or(SwapError::PairNotFound)?;

        // Calculate output amount
        let output_amount = self.calculate_swap_amount(
            amount,
            pair.base_reserve,
            pair.quote_reserve,
        );

        // Check price impact
        let price_impact = self.get_price_impact(amount, pair.base_reserve);
        if price_impact > self.amm_params.price_impact_limit * 100.0 {
            return Err(SwapError::ExcessivePriceImpact);
        }

        // Update reserves
        pair.base_reserve += amount;
        pair.quote_reserve -= output_amount;

        // Update price and volume
        pair.last_price = output_amount as f64 / amount as f64;
        pair.volume_24h += amount;

        Ok(SwapResult {
            input_amount: amount,
            output_amount,
            price_impact,
            fee_paid: amount * self.amm_params.fee_tier / 10000,
        })
    }
}

pub struct SwapResult {
    pub input_amount: u64,
    pub output_amount: u64,
    pub price_impact: f64,
    pub fee_paid: u64,
}