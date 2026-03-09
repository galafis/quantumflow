//! # QuantumFlow
//!
//! High-frequency trading engine with ultra-low latency order matching,
//! real-time market data streaming, risk management, and historical
//! backtesting capabilities.
//!
//! ## Modules
//!
//! - [`engine`] -- Matching engine and order book with price-time priority
//! - [`risk`] -- Position tracking, risk limits, and circuit breaker
//! - [`connectors`] -- Exchange WebSocket connectors (Binance)
//! - [`backtest`] -- Historical backtesting with performance metrics
//! - [`utils`] -- Shared types: Order, Trade, Ticker, OrderBookSnapshot

pub mod backtest;
pub mod connectors;
pub mod engine;
pub mod risk;
pub mod utils;

pub use engine::matching::MatchingEngine;
pub use engine::orderbook::OrderBook;
pub use risk::manager::{RiskLimits, RiskManager};
pub use utils::types::*;
