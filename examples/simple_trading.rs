use quantumflow::{
    engine::matching::MatchingEngine,
    Order, OrderType, Side,
};
use rust_decimal::Decimal;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create matching engine
    let (trade_tx, mut trade_rx) = mpsc::unbounded_channel();
    let engine = MatchingEngine::new(trade_tx);

    // Spawn task to handle trades
    tokio::spawn(async move {
        while let Some(trade) = trade_rx.recv().await {
            println!(
                "Trade executed: {} @ {} qty {}",
                trade.symbol, trade.price, trade.quantity
            );
        }
    });

    // Create buy order
    let buy_order = Order::new(
        "BTCUSD".to_string(),
        Side::Buy,
        OrderType::Limit,
        Decimal::from(50000),
        Decimal::from(2),
    );

    println!("Submitting buy order...");
    let result = engine.submit_order(buy_order).await?;
    println!("Buy order status: {:?}", result.status);

    // Create sell order
    let sell_order = Order::new(
        "BTCUSD".to_string(),
        Side::Sell,
        OrderType::Limit,
        Decimal::from(50000),
        Decimal::from(2),
    );

    println!("Submitting sell order...");
    let result = engine.submit_order(sell_order).await?;
    println!("Sell order status: {:?}", result.status);

    // Get orderbook snapshot
    if let Some(snapshot) = engine.get_orderbook_snapshot("BTCUSD") {
        println!("\nOrderbook Snapshot:");
        println!("  Symbol: {}", snapshot.symbol);
        println!("  Bids: {} levels", snapshot.bids.len());
        println!("  Asks: {} levels", snapshot.asks.len());
    }

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    Ok(())
}
