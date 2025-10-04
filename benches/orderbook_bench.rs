use criterion::{black_box, criterion_group, criterion_main, Criterion};
use quantumflow::{engine::orderbook::OrderBook, Order, OrderType, Side};
use rust_decimal::Decimal;

fn orderbook_add_benchmark(c: &mut Criterion) {
    c.bench_function("orderbook_add_1000_orders", |b| {
        b.iter(|| {
            let mut book = OrderBook::new("BTCUSD".to_string());
            for i in 0..1000 {
                let order = Order::new(
                    "BTCUSD".to_string(),
                    if i % 2 == 0 { Side::Buy } else { Side::Sell },
                    OrderType::Limit,
                    Decimal::from(50000 + i),
                    Decimal::from(1),
                );
                book.add_order(order);
            }
            black_box(book);
        });
    });
}

fn orderbook_match_benchmark(c: &mut Criterion) {
    c.bench_function("orderbook_match_orders", |b| {
        b.iter(|| {
            let mut book = OrderBook::new("BTCUSD".to_string());

            // Add buy orders
            for i in 0..100 {
                let order = Order::new(
                    "BTCUSD".to_string(),
                    Side::Buy,
                    OrderType::Limit,
                    Decimal::from(50000 - i * 10),
                    Decimal::from(1),
                );
                book.add_order(order);
            }

            // Match with sell order
            let sell_order = Order::new(
                "BTCUSD".to_string(),
                Side::Sell,
                OrderType::Limit,
                Decimal::from(49000),
                Decimal::from(50),
            );

            let (_, trades) = book.match_order(sell_order);
            black_box(trades);
        });
    });
}

fn orderbook_snapshot_benchmark(c: &mut Criterion) {
    let mut book = OrderBook::new("BTCUSD".to_string());

    for i in 0..1000 {
        let order = Order::new(
            "BTCUSD".to_string(),
            if i % 2 == 0 { Side::Buy } else { Side::Sell },
            OrderType::Limit,
            Decimal::from(50000 + i),
            Decimal::from(1),
        );
        book.add_order(order);
    }

    c.bench_function("orderbook_snapshot", |b| {
        b.iter(|| {
            let snapshot = book.get_snapshot();
            black_box(snapshot);
        });
    });
}

criterion_group!(
    benches,
    orderbook_add_benchmark,
    orderbook_match_benchmark,
    orderbook_snapshot_benchmark
);
criterion_main!(benches);
