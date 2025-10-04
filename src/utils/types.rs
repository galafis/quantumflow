use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Limit,
    Market,
    StopLimit,
    StopMarket,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

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

    pub fn remaining_quantity(&self) -> Decimal {
        self.quantity - self.filled_quantity
    }

    pub fn is_fully_filled(&self) -> bool {
        self.filled_quantity >= self.quantity
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub symbol: String,
    pub bid: Decimal,
    pub ask: Decimal,
    pub last: Decimal,
    pub volume_24h: Decimal,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookLevel {
    pub price: Decimal,
    pub quantity: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookSnapshot {
    pub symbol: String,
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
    pub timestamp: DateTime<Utc>,
}
