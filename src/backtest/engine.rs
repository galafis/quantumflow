use crate::utils::types::{Order, OrderType, Side, Trade};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OHLCV {
    pub timestamp: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
}

#[derive(Debug, Clone)]
pub struct BacktestResult {
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub total_pnl: Decimal,
    pub max_drawdown: Decimal,
    pub sharpe_ratio: f64,
    pub win_rate: f64,
    pub trades: Vec<Trade>,
}

pub struct BacktestEngine {
    initial_capital: Decimal,
    current_capital: Decimal,
    position: Decimal,
    position_price: Decimal,
    trades: Vec<Trade>,
    equity_curve: Vec<Decimal>,
}

impl BacktestEngine {
    pub fn new(initial_capital: Decimal) -> Self {
        Self {
            initial_capital,
            current_capital: initial_capital,
            position: Decimal::ZERO,
            position_price: Decimal::ZERO,
            trades: Vec::new(),
            equity_curve: vec![initial_capital],
        }
    }

    pub fn execute_signal(
        &mut self,
        symbol: &str,
        side: Side,
        price: Decimal,
        quantity: Decimal,
        timestamp: DateTime<Utc>,
    ) -> Option<Trade> {
        match side {
            Side::Buy => {
                if self.position >= Decimal::ZERO {
                    // Opening or adding to long position
                    let cost = price * quantity;
                    if cost <= self.current_capital {
                        let total_cost = self.position_price * self.position + cost;
                        self.position += quantity;
                        self.position_price = total_cost / self.position;
                        self.current_capital -= cost;

                        let trade = Trade {
                            id: uuid::Uuid::new_v4(),
                            symbol: symbol.to_string(),
                            price,
                            quantity,
                            buy_order_id: uuid::Uuid::new_v4(),
                            sell_order_id: uuid::Uuid::new_v4(),
                            timestamp,
                        };

                        self.trades.push(trade.clone());
                        info!("BUY: {} @ {} qty {}", symbol, price, quantity);
                        return Some(trade);
                    }
                } else {
                    // Closing short position
                    let pnl = (self.position_price - price) * quantity;
                    self.current_capital += pnl;
                    self.position += quantity;

                    if self.position == Decimal::ZERO {
                        self.position_price = Decimal::ZERO;
                    }

                    let trade = Trade {
                        id: uuid::Uuid::new_v4(),
                        symbol: symbol.to_string(),
                        price,
                        quantity,
                        buy_order_id: uuid::Uuid::new_v4(),
                        sell_order_id: uuid::Uuid::new_v4(),
                        timestamp,
                    };

                    self.trades.push(trade.clone());
                    info!("COVER: {} @ {} qty {}, PnL: {}", symbol, price, quantity, pnl);
                    return Some(trade);
                }
            }
            Side::Sell => {
                if self.position > Decimal::ZERO {
                    // Closing long position
                    let sell_quantity = quantity.min(self.position);
                    let pnl = (price - self.position_price) * sell_quantity;
                    self.current_capital += price * sell_quantity + pnl;
                    self.position -= sell_quantity;

                    if self.position == Decimal::ZERO {
                        self.position_price = Decimal::ZERO;
                    }

                    let trade = Trade {
                        id: uuid::Uuid::new_v4(),
                        symbol: symbol.to_string(),
                        price,
                        quantity: sell_quantity,
                        buy_order_id: uuid::Uuid::new_v4(),
                        sell_order_id: uuid::Uuid::new_v4(),
                        timestamp,
                    };

                    self.trades.push(trade.clone());
                    info!("SELL: {} @ {} qty {}, PnL: {}", symbol, price, sell_quantity, pnl);
                    return Some(trade);
                } else {
                    // Opening short position
                    self.position -= quantity;
                    self.position_price = price;
                    self.current_capital += price * quantity;

                    let trade = Trade {
                        id: uuid::Uuid::new_v4(),
                        symbol: symbol.to_string(),
                        price,
                        quantity,
                        buy_order_id: uuid::Uuid::new_v4(),
                        sell_order_id: uuid::Uuid::new_v4(),
                        timestamp,
                    };

                    self.trades.push(trade.clone());
                    info!("SHORT: {} @ {} qty {}", symbol, price, quantity);
                    return Some(trade);
                }
            }
        }

        None
    }

    pub fn update_equity(&mut self, current_price: Decimal) {
        let position_value = if self.position > Decimal::ZERO {
            self.position * current_price
        } else {
            self.position * (Decimal::from(2) * self.position_price - current_price)
        };

        let total_equity = self.current_capital + position_value;
        self.equity_curve.push(total_equity);
    }

    pub fn get_results(&self) -> BacktestResult {
        let total_pnl = self.current_capital - self.initial_capital;

        let mut winning_trades = 0;
        let mut losing_trades = 0;
        let mut pnls = Vec::new();

        // Calculate per-trade PnL (simplified)
        for i in (0..self.trades.len()).step_by(2) {
            if i + 1 < self.trades.len() {
                let entry = &self.trades[i];
                let exit = &self.trades[i + 1];

                let pnl = if entry.price < exit.price {
                    (exit.price - entry.price) * entry.quantity
                } else {
                    (entry.price - exit.price) * entry.quantity
                };

                pnls.push(pnl);

                if pnl > Decimal::ZERO {
                    winning_trades += 1;
                } else {
                    losing_trades += 1;
                }
            }
        }

        let total_trades = winning_trades + losing_trades;
        let win_rate = if total_trades > 0 {
            winning_trades as f64 / total_trades as f64
        } else {
            0.0
        };

        // Calculate max drawdown
        let mut max_drawdown = Decimal::ZERO;
        let mut peak = self.initial_capital;

        for &equity in &self.equity_curve {
            if equity > peak {
                peak = equity;
            }
            let drawdown = (peak - equity) / peak;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }

        // Calculate Sharpe ratio (simplified)
        let returns: Vec<f64> = self
            .equity_curve
            .windows(2)
            .map(|w| {
                let ret = (w[1] - w[0]) / w[0];
                ret.to_string().parse::<f64>().unwrap_or(0.0)
            })
            .collect();

        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns
            .iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>()
            / returns.len() as f64;
        let std_dev = variance.sqrt();

        let sharpe_ratio = if std_dev > 0.0 {
            mean_return / std_dev * (252.0_f64).sqrt() // Annualized
        } else {
            0.0
        };

        BacktestResult {
            total_trades,
            winning_trades,
            losing_trades,
            total_pnl,
            max_drawdown,
            sharpe_ratio,
            win_rate,
            trades: self.trades.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backtest_simple_trade() {
        let mut engine = BacktestEngine::new(Decimal::from(10000));

        engine.execute_signal(
            "BTCUSD",
            Side::Buy,
            Decimal::from(50000),
            Decimal::from(1),
            Utc::now(),
        );

        engine.execute_signal(
            "BTCUSD",
            Side::Sell,
            Decimal::from(51000),
            Decimal::from(1),
            Utc::now(),
        );

        let results = engine.get_results();
        assert!(results.total_pnl > Decimal::ZERO);
    }
}
