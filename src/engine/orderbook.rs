use crate::utils::types::{Order, OrderBookLevel, OrderBookSnapshot, Side, Trade};
use chrono::Utc;
use rust_decimal::Decimal;
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct OrderBook {
    symbol: String,
    bids: BTreeMap<Decimal, Vec<Order>>, // Price -> Orders (descending)
    asks: BTreeMap<Decimal, Vec<Order>>, // Price -> Orders (ascending)
}

impl OrderBook {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn add_order(&mut self, order: Order) {
        let price = order.price;
        match order.side {
            Side::Buy => {
                self.bids.entry(price).or_insert_with(Vec::new).push(order);
            }
            Side::Sell => {
                self.asks.entry(price).or_insert_with(Vec::new).push(order);
            }
        }
    }

    pub fn remove_order(&mut self, order_id: Uuid, side: Side) -> Option<Order> {
        let book = match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };

        let mut price_to_remove = None;
        let mut found_order = None;

        for (price, orders) in book.iter_mut() {
            if let Some(pos) = orders.iter().position(|o| o.id == order_id) {
                let order = orders.remove(pos);
                if orders.is_empty() {
                    price_to_remove = Some(*price);
                }
                found_order = Some(order);
                break;
            }
        }

        if let Some(price) = price_to_remove {
            book.remove(&price);
        }

        found_order
    }

    pub fn match_order(&mut self, mut order: Order) -> (Order, Vec<Trade>) {
        let mut trades = Vec::new();

        let opposite_book = match order.side {
            Side::Buy => &mut self.asks,
            Side::Sell => &mut self.bids,
        };

        let prices_to_match: Vec<Decimal> = match order.side {
            Side::Buy => opposite_book
                .iter()
                .filter(|(price, _)| **price <= order.price)
                .map(|(price, _)| *price)
                .collect(),
            Side::Sell => opposite_book
                .iter()
                .rev()
                .filter(|(price, _)| **price >= order.price)
                .map(|(price, _)| *price)
                .collect(),
        };

        for price in prices_to_match {
            if order.is_fully_filled() {
                break;
            }

            if let Some(orders_at_price) = opposite_book.get_mut(&price) {
                let mut i = 0;
                while i < orders_at_price.len() && !order.is_fully_filled() {
                    let opposite_order = &mut orders_at_price[i];
                    let trade_quantity = order
                        .remaining_quantity()
                        .min(opposite_order.remaining_quantity());

                    // Create trade
                    let (buy_order_id, sell_order_id) = match order.side {
                        Side::Buy => (order.id, opposite_order.id),
                        Side::Sell => (opposite_order.id, order.id),
                    };

                    let trade = Trade::new(
                        self.symbol.clone(),
                        price,
                        trade_quantity,
                        buy_order_id,
                        sell_order_id,
                    );

                    trades.push(trade);

                    // Update filled quantities
                    order.filled_quantity += trade_quantity;
                    opposite_order.filled_quantity += trade_quantity;

                    if opposite_order.is_fully_filled() {
                        orders_at_price.remove(i);
                    } else {
                        i += 1;
                    }
                }

                if orders_at_price.is_empty() {
                    opposite_book.remove(&price);
                }
            }
        }

        (order, trades)
    }

    pub fn get_best_bid(&self) -> Option<Decimal> {
        self.bids.keys().next_back().copied()
    }

    pub fn get_best_ask(&self) -> Option<Decimal> {
        self.asks.keys().next().copied()
    }

    pub fn get_spread(&self) -> Option<Decimal> {
        match (self.get_best_bid(), self.get_best_ask()) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }

    pub fn get_snapshot(&self) -> OrderBookSnapshot {
        let bids: Vec<OrderBookLevel> = self
            .bids
            .iter()
            .rev()
            .take(20)
            .map(|(price, orders)| {
                let quantity = orders.iter().map(|o| o.remaining_quantity()).sum();
                OrderBookLevel {
                    price: *price,
                    quantity,
                }
            })
            .collect();

        let asks: Vec<OrderBookLevel> = self
            .asks
            .iter()
            .take(20)
            .map(|(price, orders)| {
                let quantity = orders.iter().map(|o| o.remaining_quantity()).sum();
                OrderBookLevel {
                    price: *price,
                    quantity,
                }
            })
            .collect();

        OrderBookSnapshot {
            symbol: self.symbol.clone(),
            bids,
            asks,
            timestamp: Utc::now(),
        }
    }

    pub fn get_depth(&self, side: Side, levels: usize) -> Vec<OrderBookLevel> {
        let book = match side {
            Side::Buy => &self.bids,
            Side::Sell => &self.asks,
        };

        let iter: Box<dyn Iterator<Item = (&Decimal, &Vec<Order>)>> = match side {
            Side::Buy => Box::new(book.iter().rev()),
            Side::Sell => Box::new(book.iter()),
        };

        iter.take(levels)
            .map(|(price, orders)| {
                let quantity = orders.iter().map(|o| o.remaining_quantity()).sum();
                OrderBookLevel {
                    price: *price,
                    quantity,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::types::OrderType;

    #[test]
    fn test_orderbook_add_and_match() {
        let mut book = OrderBook::new("BTCUSD".to_string());

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

        book.add_order(buy_order.clone());
        let (matched_order, trades) = book.match_order(sell_order);

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].quantity, Decimal::from(1));
        assert!(matched_order.is_fully_filled());
    }

    #[test]
    fn test_orderbook_spread() {
        let mut book = OrderBook::new("BTCUSD".to_string());

        book.add_order(Order::new(
            "BTCUSD".to_string(),
            Side::Buy,
            OrderType::Limit,
            Decimal::from(49900),
            Decimal::from(1),
        ));

        book.add_order(Order::new(
            "BTCUSD".to_string(),
            Side::Sell,
            OrderType::Limit,
            Decimal::from(50100),
            Decimal::from(1),
        ));

        assert_eq!(book.get_spread(), Some(Decimal::from(200)));
    }
}
