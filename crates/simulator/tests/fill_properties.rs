use chrono::{Duration, TimeZone, Utc};
use grand_edge_domain::{ExecutionMode, Gp, IntervalPrice, ItemId, PriceInterval, StrategyId};
use proptest::prelude::*;
use proptest::test_runner::{Config as ProptestConfig, FailurePersistence};
use rstest::rstest;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use uuid::Uuid;

use grand_edge_simulator::{
    SimulationEngine, SimulatorConfig,
    orders::{SimulatedOrderRequest, SimulatedOrderSide},
};

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

fn engine(mode: ExecutionMode) -> SimulationEngine {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let guard = runtime.enter();
    let storage = grand_edge_storage::Storage::new(
        PgPoolOptions::new().connect_lazy_with(
            PgConnectOptions::new()
                .host("localhost")
                .username("grandedge")
                .password("grandedge")
                .database("grandedge"),
        ),
    );
    drop(guard);
    std::mem::forget(runtime);
    SimulationEngine::new(
        storage,
        SimulatorConfig {
            execution_mode: mode,
            ..SimulatorConfig::default()
        },
    )
}

fn request() -> SimulatedOrderRequest {
    SimulatedOrderRequest {
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
    }
}

#[rstest]
#[case(ExecutionMode::PassiveEstimated)]
#[case(ExecutionMode::HaircutPassive)]
fn creation_bucket_never_counts_as_future_fill(#[case] mode: ExecutionMode) {
    let outcome = engine(mode).simulate_from_history(
        request(),
        &[
            row(0, 102_000, 99_000),
            row(1, 103_000, 100_001),
            row(2, 104_000, 99_000),
            row(3, 103_500, 100_500),
        ],
    );

    assert!(outcome.is_ok());
    assert_ne!(
        outcome.unwrap().entry_time,
        row(0, 102_000, 99_000).bucket_start
    );
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: None::<Box<dyn FailurePersistence>>,
        .. ProptestConfig::default()
    })]

    #[test]
    fn simulation_never_uses_future_data_before_order_creation(
        creation_bucket_low in 1_i64..99_999_i64,
        future_low in 100_001_i64..150_000_i64
    ) {
        let result = engine(ExecutionMode::PassiveEstimated).simulate_from_history(
            request(),
            &[
                row(0, 101_000, creation_bucket_low),
                row(1, 103_000, future_low),
                row(2, 103_500, future_low + 1),
            ],
        );

        prop_assert!(result.is_err());
    }
}
