use crate::utils::types::{Order, Side};
use dashmap::DashMap;
use rust_decimal::Decimal;
use std::sync::Arc;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct RiskLimits {
    pub max_position_size: Decimal,
    pub max_order_size: Decimal,
    pub max_daily_loss: Decimal,
    pub max_leverage: Decimal,
}

impl Default for RiskLimits {
    fn default() -> Self {
        Self {
            max_position_size: Decimal::from(100),
            max_order_size: Decimal::from(10),
            max_daily_loss: Decimal::from(10000),
            max_leverage: Decimal::from(5),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub quantity: Decimal,
    pub average_price: Decimal,
    pub realized_pnl: Decimal,
}

impl Position {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            quantity: Decimal::ZERO,
            average_price: Decimal::ZERO,
            realized_pnl: Decimal::ZERO,
        }
    }

    pub fn update(&mut self, side: Side, price: Decimal, quantity: Decimal) {
        match side {
            Side::Buy => {
                let total_cost = self.average_price * self.quantity + price * quantity;
                self.quantity += quantity;
                if self.quantity > Decimal::ZERO {
                    self.average_price = total_cost / self.quantity;
                }
            }
            Side::Sell => {
                if self.quantity > Decimal::ZERO {
                    let pnl = (price - self.average_price) * quantity;
                    self.realized_pnl += pnl;
                }
                self.quantity -= quantity;
                if self.quantity <= Decimal::ZERO {
                    self.quantity = Decimal::ZERO;
                    self.average_price = Decimal::ZERO;
                }
            }
        }
    }

    pub fn unrealized_pnl(&self, current_price: Decimal) -> Decimal {
        if self.quantity == Decimal::ZERO {
            return Decimal::ZERO;
        }
        (current_price - self.average_price) * self.quantity
    }
}

pub struct RiskManager {
    limits: RiskLimits,
    positions: Arc<DashMap<String, Position>>,
    daily_pnl: Arc<parking_lot::RwLock<Decimal>>,
}

impl RiskManager {
    pub fn new(limits: RiskLimits) -> Self {
        Self {
            limits,
            positions: Arc::new(DashMap::new()),
            daily_pnl: Arc::new(parking_lot::RwLock::new(Decimal::ZERO)),
        }
    }

    pub fn check_order(&self, order: &Order) -> Result<(), String> {
        // Check order size
        if order.quantity > self.limits.max_order_size {
            return Err(format!(
                "Order size {} exceeds maximum {}",
                order.quantity, self.limits.max_order_size
            ));
        }

        // Check position size
        let position = self.get_position(&order.symbol);
        let new_position_size = match order.side {
            Side::Buy => position.quantity + order.quantity,
            Side::Sell => (position.quantity - order.quantity).abs(),
        };

        if new_position_size > self.limits.max_position_size {
            return Err(format!(
                "Position size {} would exceed maximum {}",
                new_position_size, self.limits.max_position_size
            ));
        }

        // Check daily loss
        let daily_pnl = *self.daily_pnl.read();
        if daily_pnl < -self.limits.max_daily_loss {
            return Err(format!(
                "Daily loss {} exceeds maximum {}",
                daily_pnl.abs(),
                self.limits.max_daily_loss
            ));
        }

        Ok(())
    }

    pub fn update_position(&self, symbol: &str, side: Side, price: Decimal, quantity: Decimal) {
        let mut position = self
            .positions
            .entry(symbol.to_string())
            .or_insert_with(|| Position::new(symbol.to_string()));

        let old_pnl = position.realized_pnl;
        position.update(side, price, quantity);
        let pnl_change = position.realized_pnl - old_pnl;

        // Update daily PnL
        let mut daily_pnl = self.daily_pnl.write();
        *daily_pnl += pnl_change;

        info!(
            "Position updated: {} {} @ {} qty {}, PnL change: {}",
            symbol, side, price, quantity, pnl_change
        );
    }

    pub fn get_position(&self, symbol: &str) -> Position {
        self.positions
            .get(symbol)
            .map(|p| p.clone())
            .unwrap_or_else(|| Position::new(symbol.to_string()))
    }

    pub fn get_all_positions(&self) -> Vec<Position> {
        self.positions.iter().map(|entry| entry.value().clone()).collect()
    }

    pub fn get_daily_pnl(&self) -> Decimal {
        *self.daily_pnl.read()
    }

    pub fn reset_daily_pnl(&self) {
        let mut daily_pnl = self.daily_pnl.write();
        *daily_pnl = Decimal::ZERO;
        info!("Daily PnL reset");
    }

    pub fn get_total_exposure(&self) -> Decimal {
        self.positions
            .iter()
            .map(|entry| {
                let pos = entry.value();
                pos.quantity * pos.average_price
            })
            .sum()
    }

    pub fn check_circuit_breaker(&self) -> bool {
        let daily_pnl = *self.daily_pnl.read();
        if daily_pnl < -self.limits.max_daily_loss {
            warn!(
                "Circuit breaker triggered! Daily loss: {} exceeds limit: {}",
                daily_pnl.abs(),
                self.limits.max_daily_loss
            );
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_update() {
        let mut position = Position::new("BTCUSD".to_string());

        position.update(Side::Buy, Decimal::from(50000), Decimal::from(1));
        assert_eq!(position.quantity, Decimal::from(1));
        assert_eq!(position.average_price, Decimal::from(50000));

        position.update(Side::Sell, Decimal::from(51000), Decimal::from(1));
        assert_eq!(position.quantity, Decimal::ZERO);
        assert_eq!(position.realized_pnl, Decimal::from(1000));
    }

    #[test]
    fn test_risk_manager_limits() {
        let limits = RiskLimits {
            max_order_size: Decimal::from(5),
            ..Default::default()
        };

        let manager = RiskManager::new(limits);

        let order = Order::new(
            "BTCUSD".to_string(),
            Side::Buy,
            crate::utils::types::OrderType::Limit,
            Decimal::from(50000),
            Decimal::from(10),
        );

        let result = manager.check_order(&order);
        assert!(result.is_err());
    }
}
