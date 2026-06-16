use chrono::{DateTime, Utc};
use grand_edge_domain::{FeatureVector, Gp, IntervalPrice, PriceInterval};
use grand_edge_storage::Storage;
use serde_json::{Map, Value, json};

use crate::{
    FeatureEngineConfig, FeatureError, GraphFeatureContext, ItemFeatureInput,
    calculations::{
        alch_floor_distance, buy_limit_utilization, ewma_variance, high_low_volume_ratio,
        log_return, mid_price, observed_high_side_volume, observed_low_side_volume,
        observed_volume, observed_volume_reliability, price_staleness_secs, rolling_mean,
        rolling_std, spread_abs, spread_pct, spread_stability, z_score,
    },
    graph::build_graph_feature_snapshot,
};

pub const FEATURE_SET_VERSION: &str = "features_v1";
const FORBIDDEN_FEATURE_KEYS: &[&str] = &["trueLiquidity", "marketDepth", "availableQuantity"];

pub struct FeatureEngine {
    storage: Storage,
    config: FeatureEngineConfig,
}

impl FeatureEngine {
    pub fn new(storage: Storage, config: FeatureEngineConfig) -> Self {
        Self { storage, config }
    }

    pub async fn compute_latest_all_items(
        &self,
        as_of: DateTime<Utc>,
    ) -> Result<Vec<FeatureVector>, FeatureError> {
        let latest_rows = self.storage.prices().latest_snapshot().await?;
        let mut feature_vectors = Vec::with_capacity(latest_rows.len());

        for latest in latest_rows {
            let item_id = latest.item_id;
            let item = self
                .storage
                .items()
                .get_item(item_id)
                .await?
                .ok_or(FeatureError::MissingItem(latest.item_id.0))?;
            let interval_5m = self
                .storage
                .prices()
                .interval_history(
                    item_id,
                    PriceInterval::FiveMinute,
                    self.config.rolling_window_5m as i64,
                )
                .await?;
            let interval_1h = self
                .storage
                .prices()
                .interval_history(
                    item_id,
                    PriceInterval::OneHour,
                    self.config.rolling_window_1h as i64,
                )
                .await?;
            let graph_context = self.load_graph_context(item_id, as_of).await?;

            let vector = self.compute_item_features(ItemFeatureInput {
                item,
                latest,
                interval_5m,
                interval_1h,
                as_of,
                graph_context,
            })?;
            feature_vectors.push(vector);
        }

        self.storage
            .features()
            .insert_feature_vectors(&feature_vectors)
            .await?;
        Ok(feature_vectors)
    }

    pub fn compute_item_features(
        &self,
        input: ItemFeatureInput,
    ) -> Result<FeatureVector, FeatureError> {
        Self::compute_item_features_with_config(&self.config, input)
    }

    pub fn compute_item_features_with_config(
        config: &FeatureEngineConfig,
        input: ItemFeatureInput,
    ) -> Result<FeatureVector, FeatureError> {
        let latest_high = input.latest.high.map(Gp::as_i64);
        let latest_low = input.latest.low.map(Gp::as_i64);
        let mid = mid_price(latest_high, latest_low);
        let spread_abs_value = spread_abs(latest_high, latest_low);
        let spread_pct_value = spread_pct(latest_high, latest_low);

        let interval_5m = sorted_history(input.interval_5m, input.as_of);
        let interval_1h = sorted_history(input.interval_1h, input.as_of);
        let five_minute_mids = history_mids(&interval_5m);
        let hourly_mids = history_mids(&interval_1h);
        let hourly_spread_pcts = history_spread_pct(&interval_1h);
        let hourly_observed_volumes = history_observed_volumes(&interval_1h);
        let hourly_returns = squared_hourly_returns(&hourly_mids);
        let last_hour_row = interval_1h.last();

        let observed_volume_1h = last_hour_row.map(observed_volume);
        let observed_high_side_volume_1h = last_hour_row.map(observed_high_side_volume);
        let observed_low_side_volume_1h = last_hour_row.map(observed_low_side_volume);
        let high_low_volume_ratio_1h = last_hour_row.and_then(high_low_volume_ratio);
        let rolling_mean_24h = rolling_mean(&hourly_mids);
        let rolling_std_24h = rolling_std(&hourly_mids);
        let z_score_24h = mid
            .zip(rolling_mean_24h)
            .zip(rolling_std_24h)
            .and_then(|((mid, mean), std)| z_score(mid, mean, std));
        let ewma_volatility_24h = ewma_variance(&hourly_returns, config.ewma_lambda).map(f64::sqrt);
        let observed_volume_rolling_mean = rolling_mean(&hourly_observed_volumes);
        let observed_volume_rolling_std = rolling_std(&hourly_observed_volumes);
        let observed_volume_z_24h = observed_volume_1h
            .map(|value| value as f64)
            .zip(observed_volume_rolling_mean)
            .zip(observed_volume_rolling_std)
            .and_then(|((value, mean), std)| z_score(value, mean, std));
        let observed_volume_reliability_24h = observed_volume_reliability(&interval_1h);
        let spread_stability_24h = spread_stability(&hourly_spread_pcts);
        let price_staleness_secs = price_staleness_secs(&input.latest, input.as_of);
        let alch_floor_distance = alch_floor_distance(mid, input.item.high_alch.map(Gp::as_i64));
        let buy_limit = input.item.buy_limit.map(i64::from);
        let buy_limit_utilization = buy_limit_utilization(observed_volume_1h, input.item.buy_limit);

        let mut values = Map::new();
        insert_option_i64(&mut values, "spread_abs", spread_abs_value);
        insert_option_i64(&mut values, "observed_volume_1h", observed_volume_1h);
        insert_option_i64(
            &mut values,
            "observed_high_side_volume_1h",
            observed_high_side_volume_1h,
        );
        insert_option_i64(
            &mut values,
            "observed_low_side_volume_1h",
            observed_low_side_volume_1h,
        );
        insert_option_i64(&mut values, "price_staleness_secs", price_staleness_secs);
        insert_option_i64(&mut values, "buy_limit", buy_limit);
        insert_option_f64(&mut values, "mid", mid);
        insert_option_f64(&mut values, "spread_pct", spread_pct_value);
        insert_option_f64(
            &mut values,
            "return_5m",
            return_from_lookback(&five_minute_mids, 1),
        );
        insert_option_f64(
            &mut values,
            "return_1h",
            return_from_lookback(&hourly_mids, 1),
        );
        insert_option_f64(
            &mut values,
            "return_6h",
            return_from_lookback(&hourly_mids, 6),
        );
        insert_option_f64(
            &mut values,
            "return_24h",
            return_from_lookback(&hourly_mids, 24),
        );
        insert_option_f64(&mut values, "rolling_mean_24h", rolling_mean_24h);
        insert_option_f64(&mut values, "rolling_std_24h", rolling_std_24h);
        insert_option_f64(&mut values, "z_score_24h", z_score_24h);
        insert_option_f64(&mut values, "ewma_volatility_24h", ewma_volatility_24h);
        insert_option_f64(&mut values, "observed_volume_z_24h", observed_volume_z_24h);
        insert_option_f64(
            &mut values,
            "observed_volume_reliability_24h",
            observed_volume_reliability_24h,
        );
        insert_option_f64(
            &mut values,
            "high_low_volume_ratio_1h",
            high_low_volume_ratio_1h,
        );
        insert_option_f64(&mut values, "spread_stability_24h", spread_stability_24h);
        insert_option_f64(&mut values, "alch_floor_distance", alch_floor_distance);
        insert_option_f64(&mut values, "buy_limit_utilization", buy_limit_utilization);
        values.insert(
            "missing_feature_policy".to_string(),
            json!("null_when_inputs_missing"),
        );

        if let Some(graph_context) = &input.graph_context {
            let graph_context = graph_context_as_of(graph_context, input.as_of);
            let graph_snapshot = build_graph_feature_snapshot(
                return_from_lookback(&hourly_mids, 6),
                ewma_volatility_24h,
                mid,
                &graph_context,
                &config.graph,
            );
            values.extend(graph_snapshot.values);
        }

        debug_assert!(
            FORBIDDEN_FEATURE_KEYS
                .iter()
                .all(|key| !values.contains_key(*key))
        );

        Ok(FeatureVector {
            item_id: input.item.item_id,
            as_of: input.as_of,
            feature_set_version: FEATURE_SET_VERSION.to_string(),
            values,
        })
    }

    async fn load_graph_context(
        &self,
        item_id: grand_edge_domain::ItemId,
        as_of: DateTime<Utc>,
    ) -> Result<Option<GraphFeatureContext>, FeatureError> {
        let Some(graph_version) = self.config.graph_version.clone() else {
            return Ok(None);
        };

        let incoming_edges = self
            .storage
            .graph()
            .active_edges_to(&graph_version, item_id)
            .await?;
        let outgoing_edges = self
            .storage
            .graph()
            .active_edges_from(&graph_version, item_id)
            .await?;
        let sector_edges = outgoing_edges
            .iter()
            .filter(|edge| edge.edge_type == grand_edge_domain::GraphEdgeType::SameCategory)
            .cloned()
            .collect::<Vec<_>>();

        Ok(Some(GraphFeatureContext {
            graph_version,
            incoming_neighbors: self.load_neighbor_histories(incoming_edges, as_of).await?,
            outgoing_neighbors: self.load_neighbor_histories(outgoing_edges, as_of).await?,
            sector_neighbors: self.load_neighbor_histories(sector_edges, as_of).await?,
        }))
    }

    async fn load_neighbor_histories(
        &self,
        edges: Vec<grand_edge_domain::ItemGraphEdge>,
        as_of: DateTime<Utc>,
    ) -> Result<Vec<crate::NeighborPriceHistory>, FeatureError> {
        let mut rows = Vec::with_capacity(edges.len());
        for edge in edges {
            let history = self
                .storage
                .prices()
                .interval_history_before(
                    edge.to_item_id,
                    PriceInterval::OneHour,
                    self.config.rolling_window_1h as i64,
                    Some(as_of + chrono::Duration::seconds(1)),
                )
                .await?;
            rows.push(crate::NeighborPriceHistory {
                edge,
                interval_1h: history,
            });
        }

        Ok(rows)
    }
}

fn sorted_history(mut rows: Vec<IntervalPrice>, as_of: DateTime<Utc>) -> Vec<IntervalPrice> {
    rows.retain(|row| row.bucket_start <= as_of);
    rows.sort_by_key(|row| row.bucket_start);
    rows
}

fn history_mids(rows: &[IntervalPrice]) -> Vec<f64> {
    rows.iter()
        .filter_map(|row| {
            mid_price(
                row.avg_high_price.map(Gp::as_i64),
                row.avg_low_price.map(Gp::as_i64),
            )
        })
        .collect()
}

fn history_spread_pct(rows: &[IntervalPrice]) -> Vec<f64> {
    rows.iter()
        .filter_map(|row| {
            spread_pct(
                row.avg_high_price.map(Gp::as_i64),
                row.avg_low_price.map(Gp::as_i64),
            )
        })
        .collect()
}

fn history_observed_volumes(rows: &[IntervalPrice]) -> Vec<f64> {
    rows.iter().map(|row| observed_volume(row) as f64).collect()
}

fn squared_hourly_returns(hourly_mids: &[f64]) -> Vec<f64> {
    hourly_mids
        .windows(2)
        .filter_map(|window| log_return(window[1], window[0]).map(|value| value * value))
        .collect()
}

fn return_from_lookback(values: &[f64], lookback_periods: usize) -> Option<f64> {
    if values.len() <= lookback_periods {
        return None;
    }

    let current = *values.last()?;
    let previous = values.get(values.len() - 1 - lookback_periods).copied()?;
    log_return(current, previous)
}

fn insert_option_i64(values: &mut Map<String, Value>, key: &str, value: Option<i64>) {
    values.insert(
        key.to_string(),
        value.map_or(Value::Null, serde_json::Value::from),
    );
}

fn insert_option_f64(values: &mut Map<String, Value>, key: &str, value: Option<f64>) {
    values.insert(
        key.to_string(),
        value.map_or(Value::Null, serde_json::Value::from),
    );
}

fn graph_context_as_of(context: &GraphFeatureContext, as_of: DateTime<Utc>) -> GraphFeatureContext {
    GraphFeatureContext {
        graph_version: context.graph_version.clone(),
        incoming_neighbors: context
            .incoming_neighbors
            .iter()
            .map(|neighbor| crate::NeighborPriceHistory {
                edge: neighbor.edge.clone(),
                interval_1h: neighbor
                    .interval_1h
                    .iter()
                    .filter(|row| row.bucket_start <= as_of)
                    .cloned()
                    .collect(),
            })
            .collect(),
        outgoing_neighbors: context
            .outgoing_neighbors
            .iter()
            .map(|neighbor| crate::NeighborPriceHistory {
                edge: neighbor.edge.clone(),
                interval_1h: neighbor
                    .interval_1h
                    .iter()
                    .filter(|row| row.bucket_start <= as_of)
                    .cloned()
                    .collect(),
            })
            .collect(),
        sector_neighbors: context
            .sector_neighbors
            .iter()
            .map(|neighbor| crate::NeighborPriceHistory {
                edge: neighbor.edge.clone(),
                interval_1h: neighbor
                    .interval_1h
                    .iter()
                    .filter(|row| row.bucket_start <= as_of)
                    .cloned()
                    .collect(),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::{FEATURE_SET_VERSION, FeatureEngine};
    use crate::{
        FeatureEngineConfig, GRAPH_FEATURE_KEYS,
        fixtures::{feature_fixture_input, graph_feature_fixture_input},
    };

    #[tokio::test]
    async fn feature_keys_are_stable_for_v1_fixture() {
        let storage = grand_edge_storage::Storage::new(
            sqlx::postgres::PgPoolOptions::new()
                .connect_lazy("postgres://grandedge:grandedge@localhost/grandedge")
                .unwrap(),
        );
        let engine = FeatureEngine::new(storage, FeatureEngineConfig::default());
        let vector = engine
            .compute_item_features(feature_fixture_input())
            .unwrap();

        assert_eq!(vector.feature_set_version, FEATURE_SET_VERSION);
        assert!(vector.values.contains_key("observed_volume_1h"));
        assert!(!vector.values.contains_key("trueLiquidity"));
        assert!(!vector.values.contains_key("marketDepth"));
        assert!(!vector.values.contains_key("availableQuantity"));
    }

    #[tokio::test]
    async fn graph_feature_keys_are_stable_for_v1() {
        let storage = grand_edge_storage::Storage::new(
            sqlx::postgres::PgPoolOptions::new()
                .connect_lazy("postgres://grandedge:grandedge@localhost/grandedge")
                .unwrap(),
        );
        let engine = FeatureEngine::new(storage, FeatureEngineConfig::default());
        let vector = engine
            .compute_item_features(graph_feature_fixture_input())
            .unwrap();

        for key in GRAPH_FEATURE_KEYS {
            assert!(vector.values.contains_key(*key), "{key} missing");
        }
    }

    #[tokio::test]
    async fn future_rows_do_not_change_features() {
        let storage = grand_edge_storage::Storage::new(
            sqlx::postgres::PgPoolOptions::new()
                .connect_lazy("postgres://grandedge:grandedge@localhost/grandedge")
                .unwrap(),
        );
        let engine = FeatureEngine::new(storage, FeatureEngineConfig::default());
        let baseline = feature_fixture_input();
        let mut with_future = baseline.clone();
        let mut future_row = with_future.interval_1h.last().cloned().unwrap();
        future_row.bucket_start = with_future.as_of + Duration::hours(2);
        future_row.avg_high_price = Some(grand_edge_domain::Gp(9999));
        future_row.avg_low_price = Some(grand_edge_domain::Gp(1111));
        future_row.high_price_volume = 10_000;
        future_row.low_price_volume = 10_000;
        with_future.interval_1h.push(future_row);

        let baseline_vector = engine.compute_item_features(baseline).unwrap();
        let future_vector = engine.compute_item_features(with_future).unwrap();

        assert_eq!(baseline_vector.values, future_vector.values);
    }

    #[tokio::test]
    async fn graph_features_ignore_future_neighbor_rows() {
        let storage = grand_edge_storage::Storage::new(
            sqlx::postgres::PgPoolOptions::new()
                .connect_lazy("postgres://grandedge:grandedge@localhost/grandedge")
                .unwrap(),
        );
        let engine = FeatureEngine::new(storage, FeatureEngineConfig::default());
        let baseline = graph_feature_fixture_input();
        let mut with_future = baseline.clone();
        let graph_context = with_future.graph_context.as_mut().unwrap();
        let neighbor = graph_context.incoming_neighbors.first_mut().unwrap();
        let mut future_row = neighbor.interval_1h.last().cloned().unwrap();
        future_row.bucket_start = with_future.as_of + Duration::hours(4);
        future_row.avg_high_price = Some(grand_edge_domain::Gp(9999));
        future_row.avg_low_price = Some(grand_edge_domain::Gp(8888));
        neighbor.interval_1h.push(future_row);

        let baseline_vector = engine.compute_item_features(baseline).unwrap();
        let future_vector = engine.compute_item_features(with_future).unwrap();

        for key in GRAPH_FEATURE_KEYS {
            assert_eq!(
                baseline_vector.values.get(*key),
                future_vector.values.get(*key),
                "{key} changed with future neighbor data"
            );
        }
    }

    #[tokio::test]
    async fn observed_volume_z_uses_past_window_only() {
        let storage = grand_edge_storage::Storage::new(
            sqlx::postgres::PgPoolOptions::new()
                .connect_lazy("postgres://grandedge:grandedge@localhost/grandedge")
                .unwrap(),
        );
        let engine = FeatureEngine::new(storage, FeatureEngineConfig::default());
        let vector = engine
            .compute_item_features(feature_fixture_input())
            .unwrap();

        assert!(
            vector
                .values
                .get("observed_volume_z_24h")
                .and_then(|value| value.as_f64())
                .is_some()
        );
    }

    #[tokio::test]
    async fn compute_item_features_produces_deterministic_fixture_values() {
        let storage = grand_edge_storage::Storage::new(
            sqlx::postgres::PgPoolOptions::new()
                .connect_lazy("postgres://grandedge:grandedge@localhost/grandedge")
                .unwrap(),
        );
        let engine = FeatureEngine::new(storage, FeatureEngineConfig::default());
        let vector = engine
            .compute_item_features(feature_fixture_input())
            .unwrap();

        assert_eq!(
            vector
                .values
                .get("spread_abs")
                .and_then(|value| value.as_i64()),
            Some(20)
        );
        assert_eq!(
            vector
                .values
                .get("observed_volume_1h")
                .and_then(|value| value.as_i64()),
            Some(784)
        );
        assert_eq!(
            vector
                .values
                .get("buy_limit")
                .and_then(|value| value.as_i64()),
            Some(70)
        );
        assert!(
            vector
                .values
                .get("mid")
                .and_then(|value| value.as_f64())
                .is_some_and(|value| (value - 90.0).abs() < f64::EPSILON)
        );
    }
}
