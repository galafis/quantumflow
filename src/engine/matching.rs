use crate::engine::orderbook::OrderBook;
use crate::utils::types::{Order, OrderStatus, Trade};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info};
use uuid::Uuid;

pub struct MatchingEngine {
    orderbooks: Arc<DashMap<String, OrderBook>>,
    trade_sender: mpsc::UnboundedSender<Trade>,
}

impl MatchingEngine {
    pub fn new(trade_sender: mpsc::UnboundedSender<Trade>) -> Self {
        Self {
            orderbooks: Arc::new(DashMap::new()),
            trade_sender,
        }
    }

    pub fn get_or_create_orderbook(&self, symbol: &str) -> OrderBook {
        self.orderbooks
            .entry(symbol.to_string())
            .or_insert_with(|| OrderBook::new(symbol.to_string()))
            .clone()
    }

    pub async fn submit_order(&self, mut order: Order) -> anyhow::Result<Order> {
        info!(
            "Submitting order: {} {} {} @ {} qty {}",
            order.id, order.symbol, order.side, order.price, order.quantity
        );

        order.status = OrderStatus::Open;

        let symbol = order.symbol.clone();
        let mut book = self.get_or_create_orderbook(&symbol);

        let (matched_order, trades) = book.match_order(order);

        // Update order status
        let final_order = if matched_order.is_fully_filled() {
            Order {
                status: OrderStatus::Filled,
                ..matched_order
            }
        } else if matched_order.filled_quantity > rust_decimal::Decimal::ZERO {
            Order {
                status: OrderStatus::PartiallyFilled,
                ..matched_order
            }
        } else {
            matched_order
        };

        // If not fully filled, add to book
        if !final_order.is_fully_filled() {
            book.add_order(final_order.clone());
        }

        // Update orderbook
        self.orderbooks.insert(symbol, book);

        // Send trades
        for trade in trades {
            info!(
                "Trade executed: {} {} @ {} qty {}",
                trade.symbol, trade.id, trade.price, trade.quantity
            );
            if let Err(e) = self.trade_sender.send(trade) {
                error!("Failed to send trade: {}", e);
            }
        }

        Ok(final_order)
    }

    pub async fn cancel_order(&self, order_id: Uuid, symbol: &str) -> anyhow::Result<()> {
        let mut book = self
            .orderbooks
            .get_mut(symbol)
            .ok_or_else(|| anyhow::anyhow!("Orderbook not found for symbol: {}", symbol))?;

        // Try both sides
        if let Some(order) = book.remove_order(order_id, crate::utils::types::Side::Buy) {
            info!("Cancelled buy order: {}", order.id);
            return Ok(());
        }

        if let Some(order) = book.remove_order(order_id, crate::utils::types::Side::Sell) {
            info!("Cancelled sell order: {}", order.id);
            return Ok(());
        }

        Err(anyhow::anyhow!("Order not found: {}", order_id))
    }

    pub fn get_orderbook_snapshot(
        &self,
        symbol: &str,
    ) -> Option<crate::utils::types::OrderBookSnapshot> {
        self.orderbooks.get(symbol).map(|book| book.get_snapshot())
    }

    pub fn get_all_symbols(&self) -> Vec<String> {
        self.orderbooks.iter().map(|entry| entry.key().clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::types::{OrderType, Side};
    use rust_decimal::Decimal;

    #[tokio::test]
    async fn test_matching_engine_submit_and_match() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let engine = MatchingEngine::new(tx);

        let buy_order = Order::new(
            "BTCUSD".to_string(),
            Side::Buy,
            OrderType::Limit,
            Decimal::from(50000),
            Decimal::from(1),
        );

        let sell_order = Order::new(
            "BTCUSD".to_string(),
            Side::Sell,
            OrderType::Limit,
            Decimal::from(50000),
            Decimal::from(1),
        );

        let result1 = engine.submit_order(buy_order).await;
        assert!(result1.is_ok());

        let result2 = engine.submit_order(sell_order).await;
        assert!(result2.is_ok());

        // Should receive one trade
        let trade = rx.recv().await;
        assert!(trade.is_some());
    }

    #[tokio::test]
    async fn test_matching_engine_cancel() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let engine = MatchingEngine::new(tx);

        let order = Order::new(
            "BTCUSD".to_string(),
            Side::Buy,
            OrderType::Limit,
            Decimal::from(50000),
            Decimal::from(1),
        );

        let order_id = order.id;
        let result = engine.submit_order(order).await;
        assert!(result.is_ok());

        let cancel_result = engine.cancel_order(order_id, "BTCUSD").await;
        assert!(cancel_result.is_ok());
    }
}
