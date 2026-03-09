//! Core domain types shared across the trading engine.
//!
//! Defines the fundamental data structures for orders, trades,
//! market data, and order book representations.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Side of an order: buy or sell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Side::Buy => write!(f, "BUY"),
            Side::Sell => write!(f, "SELL"),
        }
    }
}

/// Type of order determining execution semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    /// Limit order: executes at the specified price or better.
    Limit,
    /// Market order: executes immediately at the best available price.
    Market,
    /// Stop-limit order: becomes a limit order when the stop price is reached.
    StopLimit,
    /// Stop-market order: becomes a market order when the stop price is reached.
    StopMarket,
}

/// Lifecycle status of an order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

/// Represents a trading order with all metadata required for matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub price: Decimal,
    pub quantity: Decimal,
    pub filled_quantity: Decimal,
    pub status: OrderStatus,
    pub timestamp: DateTime<Utc>,
    pub client_id: Option<String>,
}

impl Order {
    /// Creates a new order with a random UUID and `Pending` status.
    pub fn new(
        symbol: String,
        side: Side,
        order_type: OrderType,
        price: Decimal,
        quantity: Decimal,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            symbol,
            side,
            order_type,
            price,
            quantity,
            filled_quantity: Decimal::ZERO,
            status: OrderStatus::Pending,
            timestamp: Utc::now(),
            client_id: None,
        }
    }

    /// Returns the quantity that has not yet been filled.
    pub fn remaining_quantity(&self) -> Decimal {
        self.quantity - self.filled_quantity
    }

    /// Returns `true` if the order has been completely filled.
    pub fn is_fully_filled(&self) -> bool {
        self.filled_quantity >= self.quantity
    }
}

/// Represents a completed trade between a buyer and a seller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: Uuid,
    pub symbol: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub buy_order_id: Uuid,
    pub sell_order_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

impl Trade {
    /// Creates a new trade record with a random UUID.
    pub fn new(
        symbol: String,
        price: Decimal,
        quantity: Decimal,
        buy_order_id: Uuid,
        sell_order_id: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            symbol,
            price,
            quantity,
            buy_order_id,
            sell_order_id,
            timestamp: Utc::now(),
        }
    }
}

/// Real-time ticker data from an exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub symbol: String,
    pub bid: Decimal,
    pub ask: Decimal,
    pub last: Decimal,
    pub volume_24h: Decimal,
    pub timestamp: DateTime<Utc>,
}

/// A single price level in an order book with aggregated quantity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookLevel {
    pub price: Decimal,
    pub quantity: Decimal,
}

/// Point-in-time snapshot of an order book with top bid/ask levels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookSnapshot {
    pub symbol: String,
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
    pub timestamp: DateTime<Utc>,
}
