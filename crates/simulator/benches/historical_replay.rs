use chrono::{Duration, TimeZone, Utc};
use criterion::{Criterion, criterion_group, criterion_main};
use grand_edge_domain::{ExecutionMode, Gp, IntervalPrice, ItemId, PriceInterval, StrategyId};
use grand_edge_simulator::{
    SimulationEngine, SimulatorConfig,
    orders::{SimulatedOrderRequest, SimulatedOrderSide},
};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

fn row(hours_after_creation: i64, high: i64, low: i64) -> IntervalPrice {
    IntervalPrice {
        item_id: ItemId(4151),
        bucket_start: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap()
            + Duration::hours(hours_after_creation),
        interval: PriceInterval::OneHour,
        avg_high_price: Some(Gp(high)),
        high_price_volume: 250,
        avg_low_price: Some(Gp(low)),
        low_price_volume: 170,
    }
}

fn historical_replay(c: &mut Criterion) {
    let storage = grand_edge_storage::Storage::new(
        PgPoolOptions::new()
            .connect_lazy("postgres://grandedge:grandedge@localhost/grandedge")
            .unwrap(),
    );
    let engine = SimulationEngine::new(
        storage,
        SimulatorConfig {
            execution_mode: ExecutionMode::PassiveEstimated,
            ..SimulatorConfig::default()
        },
    );
    let request = SimulatedOrderRequest {
        run_id: Uuid::new_v4(),
        strategy_id: StrategyId("spread_edge_v1".to_string()).0,
        model_version: "v1".to_string(),
        item_id: 4151,
        created_at: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
        side: SimulatedOrderSide::Buy,
        quantity: 20,
        limit_price: Some(100_000),
        target_exit: Some(103_000),
        stop_loss: Some(99_000),
        horizon_secs: 21_600,
    };
    let history = vec![
        row(1, 101_000, 100_000),
        row(2, 103_500, 100_500),
        row(3, 104_000, 99_500),
        row(4, 103_000, 100_000),
    ];

    c.bench_function("historical_replay_fixture", |b| {
        b.iter(|| {
            engine
                .simulate_from_history(request.clone(), &history)
                .unwrap()
        })
    });
}

criterion_group!(benches, historical_replay);
criterion_main!(benches);
