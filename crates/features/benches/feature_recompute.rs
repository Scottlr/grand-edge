use criterion::{Criterion, criterion_group, criterion_main};
use grand_edge_features::{FeatureEngine, FeatureEngineConfig, fixtures::feature_fixture_input};
use sqlx::postgres::PgPoolOptions;

fn feature_recompute(c: &mut Criterion) {
    let storage = grand_edge_storage::Storage::new(
        PgPoolOptions::new()
            .connect_lazy("postgres://grandedge:grandedge@localhost/grandedge")
            .unwrap(),
    );
    let engine = FeatureEngine::new(storage, FeatureEngineConfig::default());
    let input = feature_fixture_input();

    c.bench_function("feature_recompute_fixture", |b| {
        b.iter(|| engine.compute_item_features(input.clone()).unwrap())
    });
}

criterion_group!(benches, feature_recompute);
criterion_main!(benches);
