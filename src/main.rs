use clap::{Parser, Subcommand};
use quantumflow::{
    backtest::engine::BacktestEngine,
    connectors::binance::BinanceConnector,
    engine::matching::MatchingEngine,
    risk::manager::{RiskLimits, RiskManager},
    Order, OrderType, Side,
};
use rust_decimal::Decimal;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, Level};
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "QuantumFlow")]
#[command(about = "High-Frequency Trading Engine", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the matching engine
    Match {
        /// Trading symbol
        #[arg(short, long, default_value = "BTCUSD")]
        symbol: String,
    },
    /// Stream market data from Binance
    Stream {
        /// Trading symbol
        #[arg(short, long, default_value = "btcusdt")]
        symbol: String,
        /// Stream type (ticker or orderbook)
        #[arg(short, long, default_value = "ticker")]
        stream_type: String,
    },
    /// Run backtest
    Backtest {
        /// CSV file with historical data
        #[arg(short, long)]
        file: String,
    },
    /// Run demo trading
    Demo,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Match { symbol } => {
            run_matching_engine(&symbol).await?;
        }
        Commands::Stream { symbol, stream_type } => {
            run_stream(&symbol, &stream_type).await?;
        }
        Commands::Backtest { file } => {
            run_backtest(&file).await?;
        }
        Commands::Demo => {
            run_demo().await?;
        }
    }

    Ok(())
}

async fn run_matching_engine(symbol: &str) -> anyhow::Result<()> {
    info!("Starting matching engine for {}", symbol);

    let (trade_tx, mut trade_rx) = mpsc::unbounded_channel();
    let engine = Arc::new(MatchingEngine::new(trade_tx));

    // Spawn task to handle trades
    tokio::spawn(async move {
        while let Some(trade) = trade_rx.recv().await {
            info!(
                "Trade: {} {} @ {} qty {}",
                trade.symbol, trade.id, trade.price, trade.quantity
            );
        }
    });

    // Submit sample orders
    let buy_order = Order::new(
        symbol.to_string(),
        Side::Buy,
        OrderType::Limit,
        Decimal::from(50000),
        Decimal::from(1),
    );

    let sell_order = Order::new(
        symbol.to_string(),
        Side::Sell,
        OrderType::Limit,
        Decimal::from(50000),
        Decimal::from(1),
    );

    engine.submit_order(buy_order).await?;
    engine.submit_order(sell_order).await?;

    // Get orderbook snapshot
    if let Some(snapshot) = engine.get_orderbook_snapshot(symbol) {
        info!("Orderbook snapshot:");
        info!("  Bids: {} levels", snapshot.bids.len());
        info!("  Asks: {} levels", snapshot.asks.len());
    }

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    Ok(())
}

async fn run_stream(symbol: &str, stream_type: &str) -> anyhow::Result<()> {
    info!("Starting {} stream for {}", stream_type, symbol);

    let connector = BinanceConnector::new();

    match stream_type {
        "ticker" => {
            connector
                .stream_ticker(symbol, |ticker| {
                    info!(
                        "Ticker: {} | Bid: {} | Ask: {} | Last: {}",
                        ticker.symbol, ticker.bid, ticker.ask, ticker.last
                    );
                })
                .await?;
        }
        "orderbook" => {
            connector
                .stream_orderbook(symbol, |snapshot| {
                    if let (Some(best_bid), Some(best_ask)) = (
                        snapshot.bids.first(),
                        snapshot.asks.first(),
                    ) {
                        info!(
                            "OrderBook: {} | Best Bid: {} | Best Ask: {} | Spread: {}",
                            snapshot.symbol,
                            best_bid.price,
                            best_ask.price,
                            best_ask.price - best_bid.price
                        );
                    }
                })
                .await?;
        }
        _ => {
            eprintln!("Unknown stream type: {}", stream_type);
        }
    }

    Ok(())
}

async fn run_backtest(file: &str) -> anyhow::Result<()> {
    info!("Running backtest with data from {}", file);

    let mut engine = BacktestEngine::new(Decimal::from(100000));

    // Load sample data (simplified)
    let prices = vec![
        Decimal::from(50000),
        Decimal::from(50500),
        Decimal::from(51000),
        Decimal::from(50800),
        Decimal::from(51200),
    ];

    for (i, price) in prices.iter().enumerate() {
        if i % 2 == 0 {
            engine.execute_signal(
                "BTCUSD",
                Side::Buy,
                *price,
                Decimal::from(1),
                chrono::Utc::now(),
            );
        } else {
            engine.execute_signal(
                "BTCUSD",
                Side::Sell,
                *price,
                Decimal::from(1),
                chrono::Utc::now(),
            );
        }
        engine.update_equity(*price);
    }

    let results = engine.get_results();

    info!("Backtest Results:");
    info!("  Total Trades: {}", results.total_trades);
    info!("  Winning Trades: {}", results.winning_trades);
    info!("  Losing Trades: {}", results.losing_trades);
    info!("  Total PnL: {}", results.total_pnl);
    info!("  Max Drawdown: {:.2}%", results.max_drawdown * Decimal::from(100));
    info!("  Sharpe Ratio: {:.2}", results.sharpe_ratio);
    info!("  Win Rate: {:.2}%", results.win_rate * 100.0);

    Ok(())
}

async fn run_demo() -> anyhow::Result<()> {
    info!("Running demo trading simulation");

    let (trade_tx, mut trade_rx) = mpsc::unbounded_channel();
    let engine = Arc::new(MatchingEngine::new(trade_tx));
    let risk_manager = Arc::new(RiskManager::new(RiskLimits::default()));

    // Spawn task to handle trades
    let rm = risk_manager.clone();
    tokio::spawn(async move {
        while let Some(trade) = trade_rx.recv().await {
            info!(
                "Trade executed: {} @ {} qty {}",
                trade.symbol, trade.price, trade.quantity
            );

            // Update risk manager
            let side = if trade.buy_order_id < trade.sell_order_id {
                Side::Buy
            } else {
                Side::Sell
            };
            rm.update_position(&trade.symbol, side, trade.price, trade.quantity);
        }
    });

    // Submit demo orders
    for i in 0..5 {
        let price = Decimal::from(50000 + i * 100);

        let buy_order = Order::new(
            "BTCUSD".to_string(),
            Side::Buy,
            OrderType::Limit,
            price,
            Decimal::from(1),
        );

        // Check risk
        if let Err(e) = risk_manager.check_order(&buy_order) {
            info!("Order rejected by risk manager: {}", e);
            continue;
        }

        engine.submit_order(buy_order).await?;

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        let sell_order = Order::new(
            "BTCUSD".to_string(),
            Side::Sell,
            OrderType::Limit,
            price + Decimal::from(200),
            Decimal::from(1),
        );

        engine.submit_order(sell_order).await?;

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    // Display risk metrics
    info!("Risk Metrics:");
    info!("  Daily PnL: {}", risk_manager.get_daily_pnl());
    info!("  Total Exposure: {}", risk_manager.get_total_exposure());
    info!("  Circuit Breaker: {}", risk_manager.check_circuit_breaker());

    for position in risk_manager.get_all_positions() {
        info!(
            "  Position: {} | Qty: {} | Avg Price: {} | PnL: {}",
            position.symbol, position.quantity, position.average_price, position.realized_pnl
        );
    }

    Ok(())
}
