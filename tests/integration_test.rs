use quantumflow::{
    backtest::engine::BacktestEngine,
    engine::matching::MatchingEngine,
    risk::manager::{RiskLimits, RiskManager},
    Order, OrderStatus, OrderType, Side,
};
use rust_decimal::Decimal;
use tokio::sync::mpsc;

#[tokio::test]
async fn test_full_trading_flow() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let engine = MatchingEngine::new(tx);
    let risk_manager = RiskManager::new(RiskLimits::default());

    // Create and submit buy order
    let buy_order = Order::new(
        "BTCUSD".to_string(),
        Side::Buy,
        OrderType::Limit,
        Decimal::from(50000),
        Decimal::from(1),
    );

    assert!(risk_manager.check_order(&buy_order).is_ok());

    let result = engine.submit_order(buy_order).await;
    assert!(result.is_ok());

    // Create and submit sell order
    let sell_order = Order::new(
        "BTCUSD".to_string(),
        Side::Sell,
        OrderType::Limit,
        Decimal::from(50000),
        Decimal::from(1),
    );

    let result = engine.submit_order(sell_order).await;
    assert!(result.is_ok());

    // Verify trade was executed
    let trade = rx.recv().await;
    assert!(trade.is_some());

    let trade = trade.unwrap();
    assert_eq!(trade.symbol, "BTCUSD");
    assert_eq!(trade.price, Decimal::from(50000));
    assert_eq!(trade.quantity, Decimal::from(1));
}

#[tokio::test]
async fn test_risk_manager_limits() {
    let limits = RiskLimits {
        max_order_size: Decimal::from(5),
        max_position_size: Decimal::from(10),
        ..Default::default()
    };

    let risk_manager = RiskManager::new(limits);

    // Order exceeds max size
    let large_order = Order::new(
        "BTCUSD".to_string(),
        Side::Buy,
        OrderType::Limit,
        Decimal::from(50000),
        Decimal::from(10),
    );

    assert!(risk_manager.check_order(&large_order).is_err());

    // Valid order
    let valid_order = Order::new(
        "BTCUSD".to_string(),
        Side::Buy,
        OrderType::Limit,
        Decimal::from(50000),
        Decimal::from(3),
    );

    assert!(risk_manager.check_order(&valid_order).is_ok());
}

#[tokio::test]
async fn test_orderbook_snapshot() {
    let (tx, _rx) = mpsc::unbounded_channel();
    let engine = MatchingEngine::new(tx);

    // Add orders
    for i in 0..10 {
        let buy_order = Order::new(
            "BTCUSD".to_string(),
            Side::Buy,
            OrderType::Limit,
            Decimal::from(50000 - i * 100),
            Decimal::from(1),
        );
        engine.submit_order(buy_order).await.unwrap();

        let sell_order = Order::new(
            "BTCUSD".to_string(),
            Side::Sell,
            OrderType::Limit,
            Decimal::from(51000 + i * 100),
            Decimal::from(1),
        );
        engine.submit_order(sell_order).await.unwrap();
    }

    // Get snapshot
    let snapshot = engine.get_orderbook_snapshot("BTCUSD");
    assert!(snapshot.is_some());

    let snapshot = snapshot.unwrap();
    assert_eq!(snapshot.symbol, "BTCUSD");
    assert!(!snapshot.bids.is_empty());
    assert!(!snapshot.asks.is_empty());
}

#[tokio::test]
async fn test_partial_fill() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let engine = MatchingEngine::new(tx);

    // Submit a buy order for 5 units
    let buy_order = Order::new(
        "BTCUSD".to_string(),
        Side::Buy,
        OrderType::Limit,
        Decimal::from(50000),
        Decimal::from(5),
    );
    engine.submit_order(buy_order).await.unwrap();

    // Submit a sell order for only 2 units at the same price
    let sell_order = Order::new(
        "BTCUSD".to_string(),
        Side::Sell,
        OrderType::Limit,
        Decimal::from(50000),
        Decimal::from(2),
    );
    let result = engine.submit_order(sell_order).await.unwrap();
    assert_eq!(result.status, OrderStatus::Filled);
    assert_eq!(result.filled_quantity, Decimal::from(2));

    // Verify trade quantity matches partial fill
    let trade = rx.recv().await.unwrap();
    assert_eq!(trade.quantity, Decimal::from(2));

    // Remaining 3 units should still be on the book
    let snapshot = engine.get_orderbook_snapshot("BTCUSD").unwrap();
    assert_eq!(snapshot.bids.len(), 1);
    assert_eq!(snapshot.bids[0].quantity, Decimal::from(3));
}

#[tokio::test]
async fn test_order_cancellation_flow() {
    let (tx, _rx) = mpsc::unbounded_channel();
    let engine = MatchingEngine::new(tx);

    // Submit a buy order
    let order = Order::new(
        "BTCUSD".to_string(),
        Side::Buy,
        OrderType::Limit,
        Decimal::from(49000),
        Decimal::from(1),
    );
    let order_id = order.id;
    engine.submit_order(order).await.unwrap();

    // Verify it is on the book
    let snapshot = engine.get_orderbook_snapshot("BTCUSD").unwrap();
    assert_eq!(snapshot.bids.len(), 1);

    // Cancel the order
    engine.cancel_order(order_id, "BTCUSD").await.unwrap();

    // Verify the book is now empty
    let snapshot = engine.get_orderbook_snapshot("BTCUSD").unwrap();
    assert!(snapshot.bids.is_empty());
}

#[tokio::test]
async fn test_multi_symbol_routing() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let engine = MatchingEngine::new(tx);

    // Submit orders for BTCUSD
    let btc_buy = Order::new(
        "BTCUSD".to_string(),
        Side::Buy,
        OrderType::Limit,
        Decimal::from(50000),
        Decimal::from(1),
    );
    engine.submit_order(btc_buy).await.unwrap();

    // Submit orders for ETHUSD
    let eth_buy = Order::new(
        "ETHUSD".to_string(),
        Side::Buy,
        OrderType::Limit,
        Decimal::from(3000),
        Decimal::from(10),
    );
    engine.submit_order(eth_buy).await.unwrap();

    // Verify both symbols are tracked
    let symbols = engine.get_all_symbols();
    assert!(symbols.contains(&"BTCUSD".to_string()));
    assert!(symbols.contains(&"ETHUSD".to_string()));

    // A sell on BTCUSD should not affect ETHUSD
    let btc_sell = Order::new(
        "BTCUSD".to_string(),
        Side::Sell,
        OrderType::Limit,
        Decimal::from(50000),
        Decimal::from(1),
    );
    engine.submit_order(btc_sell).await.unwrap();

    let trade = rx.recv().await.unwrap();
    assert_eq!(trade.symbol, "BTCUSD");

    // ETHUSD order book should still have the buy order
    let eth_snapshot = engine.get_orderbook_snapshot("ETHUSD").unwrap();
    assert_eq!(eth_snapshot.bids.len(), 1);
    assert_eq!(eth_snapshot.bids[0].quantity, Decimal::from(10));
}

#[tokio::test]
async fn test_risk_manager_position_tracking() {
    let risk_manager = RiskManager::new(RiskLimits::default());

    // Buy 5 units at 50000
    risk_manager.update_position("BTCUSD", Side::Buy, Decimal::from(50000), Decimal::from(5));

    let position = risk_manager.get_position("BTCUSD");
    assert_eq!(position.quantity, Decimal::from(5));
    assert_eq!(position.average_price, Decimal::from(50000));

    // Sell 5 units at 51000 => realized PnL = (51000 - 50000) * 5 = 5000
    risk_manager.update_position("BTCUSD", Side::Sell, Decimal::from(51000), Decimal::from(5));

    let position = risk_manager.get_position("BTCUSD");
    assert_eq!(position.quantity, Decimal::ZERO);
    assert_eq!(position.realized_pnl, Decimal::from(5000));

    // Daily PnL should reflect the gain
    assert_eq!(risk_manager.get_daily_pnl(), Decimal::from(5000));
}

#[tokio::test]
async fn test_circuit_breaker() {
    let limits = RiskLimits {
        max_daily_loss: Decimal::from(1000),
        ..Default::default()
    };
    let risk_manager = RiskManager::new(limits);

    // Simulate a losing trade: buy at 50000, sell at 49000 => PnL = -1000
    risk_manager.update_position("BTCUSD", Side::Buy, Decimal::from(50000), Decimal::from(1));
    risk_manager.update_position("BTCUSD", Side::Sell, Decimal::from(49000), Decimal::from(1));

    // PnL is -1000, which equals the limit
    assert_eq!(risk_manager.get_daily_pnl(), Decimal::from(-1000));

    // Simulate another loss to breach the limit
    risk_manager.update_position("BTCUSD", Side::Buy, Decimal::from(48000), Decimal::from(1));
    risk_manager.update_position("BTCUSD", Side::Sell, Decimal::from(47000), Decimal::from(1));

    // Circuit breaker should trigger (daily loss > max_daily_loss)
    assert!(risk_manager.check_circuit_breaker());
}

#[tokio::test]
async fn test_daily_pnl_reset() {
    let risk_manager = RiskManager::new(RiskLimits::default());

    risk_manager.update_position("BTCUSD", Side::Buy, Decimal::from(50000), Decimal::from(1));
    risk_manager.update_position("BTCUSD", Side::Sell, Decimal::from(51000), Decimal::from(1));

    assert!(risk_manager.get_daily_pnl() != Decimal::ZERO);

    risk_manager.reset_daily_pnl();
    assert_eq!(risk_manager.get_daily_pnl(), Decimal::ZERO);
}

#[test]
fn test_backtest_engine_basic() {
    let mut engine = BacktestEngine::new(Decimal::from(100000));

    // Buy at 50000, sell at 51000 => profit
    engine.execute_signal(
        "BTCUSD",
        Side::Buy,
        Decimal::from(50000),
        Decimal::from(1),
        chrono::Utc::now(),
    );
    engine.update_equity(Decimal::from(50000));

    engine.execute_signal(
        "BTCUSD",
        Side::Sell,
        Decimal::from(51000),
        Decimal::from(1),
        chrono::Utc::now(),
    );
    engine.update_equity(Decimal::from(51000));

    let results = engine.get_results();
    assert_eq!(results.total_trades, 1);
    assert!(results.total_pnl > Decimal::ZERO);
    assert!(results.win_rate >= 0.0);
    assert!(results.sharpe_ratio.is_finite());
}

#[test]
fn test_backtest_engine_multiple_trades() {
    let mut engine = BacktestEngine::new(Decimal::from(100000));

    let prices = vec![
        (Side::Buy, Decimal::from(50000)),
        (Side::Sell, Decimal::from(50500)),
        (Side::Buy, Decimal::from(51000)),
        (Side::Sell, Decimal::from(50800)),
    ];

    for (side, price) in &prices {
        engine.execute_signal("BTCUSD", *side, *price, Decimal::from(1), chrono::Utc::now());
        engine.update_equity(*price);
    }

    let results = engine.get_results();
    assert_eq!(results.total_trades, 2);
    assert_eq!(results.winning_trades + results.losing_trades, 2);
}

#[test]
fn test_backtest_max_drawdown() {
    let mut engine = BacktestEngine::new(Decimal::from(100000));

    // Buy and lose money to create drawdown
    engine.execute_signal(
        "BTCUSD",
        Side::Buy,
        Decimal::from(50000),
        Decimal::from(1),
        chrono::Utc::now(),
    );
    engine.update_equity(Decimal::from(50000));

    engine.execute_signal(
        "BTCUSD",
        Side::Sell,
        Decimal::from(45000),
        Decimal::from(1),
        chrono::Utc::now(),
    );
    engine.update_equity(Decimal::from(45000));

    let results = engine.get_results();
    assert!(results.max_drawdown > Decimal::ZERO);
}
