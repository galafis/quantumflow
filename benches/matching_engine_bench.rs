use criterion::{black_box, criterion_group, criterion_main, Criterion};
use quantumflow::{engine::matching::MatchingEngine, Order, OrderType, Side};
use rust_decimal::Decimal;
use tokio::sync::mpsc;

fn matching_engine_submit_benchmark(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("matching_engine_submit_100_orders", |b| {
        b.to_async(&runtime).iter(|| async {
            let (tx, _rx) = mpsc::unbounded_channel();
            let engine = MatchingEngine::new(tx);

            for i in 0..100 {
                let order = Order::new(
                    "BTCUSD".to_string(),
                    if i % 2 == 0 { Side::Buy } else { Side::Sell },
                    OrderType::Limit,
                    Decimal::from(50000),
                    Decimal::from(1),
                );
                let _ = engine.submit_order(order).await;
            }

            black_box(engine);
        });
    });
}

criterion_group!(benches, matching_engine_submit_benchmark);
criterion_main!(benches);
