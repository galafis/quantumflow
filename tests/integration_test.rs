use quantumflow::{
    engine::matching::MatchingEngine,
    risk::manager::{RiskLimits, RiskManager},
    Order, OrderType, Side,
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
