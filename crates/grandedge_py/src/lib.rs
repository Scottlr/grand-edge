use grand_edge_domain::{IntervalPrice, ItemId, MarketRules};
use grand_edge_features::{FeatureEngine, FeatureEngineConfig, ItemFeatureInput};
use grand_edge_simulator::{SimulatedOrderRequest, SimulationEngine, SimulatorConfig};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyModule;
use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
enum BindingError {
    #[error("invalid JSON request: {0}")]
    InvalidJson(String),
    #[error("feature calculation failed: {0}")]
    Feature(String),
    #[error("simulation failed: {0}")]
    Simulation(String),
}

#[derive(Debug, Clone, Deserialize)]
struct FeatureRequestEnvelope {
    input: ItemFeatureInput,
    #[serde(default)]
    config: Option<FeatureEngineConfig>,
}

#[derive(Debug, Clone, Deserialize)]
struct SimulationRequestEnvelope {
    request: SimulatedOrderRequest,
    #[serde(default)]
    config: Option<SimulatorConfig>,
}

#[pyfunction]
fn tax_for_sale(market_rules_json: &str, item_id: i64, sell_price_gp: i64) -> PyResult<i64> {
    let rules: MarketRules = parse_json(market_rules_json)?;
    Ok(rules
        .tax_for_sale(ItemId(item_id), grand_edge_domain::Gp(sell_price_gp))
        .as_i64())
}

#[pyfunction]
fn spread_features_from_json(feature_input_json: &str) -> PyResult<String> {
    let envelope: FeatureRequestEnvelope = parse_json(feature_input_json)?;
    let config = envelope.config.unwrap_or_default();
    let output = FeatureEngine::compute_item_features_with_config(&config, envelope.input)
        .map_err(|error| map_runtime_error(BindingError::Feature(error.to_string())))?;
    serialize_json(&output)
}

#[pyfunction]
fn simulate_order_from_json(request_json: &str, history_json: &str) -> PyResult<String> {
    let envelope: SimulationRequestEnvelope = parse_json(request_json)?;
    let history: Vec<IntervalPrice> = parse_json(history_json)?;
    let config = envelope.config.unwrap_or_default();
    let output =
        SimulationEngine::simulate_from_history_with_config(&config, envelope.request, &history)
            .map_err(|error| map_runtime_error(BindingError::Simulation(error.to_string())))?;
    serialize_json(&output)
}

fn parse_json<T>(payload: &str) -> PyResult<T>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_str(payload).map_err(|error| {
        PyValueError::new_err(BindingError::InvalidJson(error.to_string()).to_string())
    })
}

fn serialize_json<T>(value: &T) -> PyResult<String>
where
    T: serde::Serialize,
{
    serde_json::to_string(value)
        .map_err(|error| PyRuntimeError::new_err(format!("failed to serialize output: {error}")))
}

fn map_runtime_error(error: BindingError) -> PyErr {
    PyRuntimeError::new_err(error.to_string())
}

#[pymodule]
fn grandedge_py(_py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(tax_for_sale, module)?)?;
    module.add_function(wrap_pyfunction!(spread_features_from_json, module)?)?;
    module.add_function(wrap_pyfunction!(simulate_order_from_json, module)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use grand_edge_domain::ExecutionMode;
    use grand_edge_features::fixtures::feature_fixture_input;
    use grand_edge_simulator::PaperBetOutcome;
    use grand_edge_simulator::{SimulatedOrderSide, SimulatedOrderStatus};
    use serde_json::json;
    use uuid::Uuid;

    use super::{spread_features_from_json, tax_for_sale};

    #[test]
    fn invalid_feature_json_returns_value_error() {
        pyo3::prepare_freethreaded_python();
        let error = spread_features_from_json("{not json").unwrap_err();
        assert!(error.to_string().contains("invalid JSON request"));
    }

    #[test]
    fn tax_fixture_matches_goal() {
        let payload = serde_json::to_string(&grand_edge_domain::MarketRules::default()).unwrap();
        let tax = tax_for_sale(&payload, 4151, 103_000).unwrap();
        assert_eq!(tax, 2_060);
    }

    #[test]
    fn spread_feature_fixture_matches_rust_engine() {
        let payload = json!({
            "input": feature_fixture_input(),
        });
        let output = spread_features_from_json(&serde_json::to_string(&payload).unwrap()).unwrap();
        let parsed: grand_edge_domain::FeatureVector = serde_json::from_str(&output).unwrap();

        assert_eq!(
            parsed
                .values
                .get("spread_abs")
                .and_then(|value| value.as_i64()),
            Some(20)
        );
        assert_eq!(
            parsed
                .values
                .get("observed_volume_1h")
                .and_then(|value| value.as_i64()),
            Some(784)
        );
    }

    #[test]
    fn simulate_order_rejects_creation_bucket_for_passive_mode() {
        let request_payload = json!({
            "config": {
                "execution_mode": ExecutionMode::PassiveEstimated,
                "market_rules": grand_edge_domain::MarketRules::default(),
                "participation_rate": 0.05,
                "confidence_haircut": 0.5,
                "default_horizon_secs": 21600,
                "emergency_exit_slippage_gp": 0,
                "worst_case_slippage_gp": 0
            },
            "request": {
                "run_id": Uuid::new_v4(),
                "strategy_id": "spread_edge_v1",
                "model_version": "v1",
                "item_id": 4151,
                "created_at": "2026-06-16T12:00:00Z",
                "side": SimulatedOrderSide::Buy,
                "quantity": 20,
                "limit_price": 100000,
                "target_exit": 103000,
                "stop_loss": 99000,
                "horizon_secs": 21600
            }
        });
        let history_payload = json!([
            {
                "item_id": 4151,
                "bucket_start": "2026-06-16T12:00:00Z",
                "interval": "one_hour",
                "avg_high_price": 102000,
                "high_price_volume": 250,
                "avg_low_price": 99000,
                "low_price_volume": 170
            },
            {
                "item_id": 4151,
                "bucket_start": "2026-06-16T13:00:00Z",
                "interval": "one_hour",
                "avg_high_price": 103000,
                "high_price_volume": 250,
                "avg_low_price": 100001,
                "low_price_volume": 170
            },
            {
                "item_id": 4151,
                "bucket_start": "2026-06-16T14:00:00Z",
                "interval": "one_hour",
                "avg_high_price": 104000,
                "high_price_volume": 250,
                "avg_low_price": 99000,
                "low_price_volume": 170
            },
            {
                "item_id": 4151,
                "bucket_start": "2026-06-16T15:00:00Z",
                "interval": "one_hour",
                "avg_high_price": 103500,
                "high_price_volume": 250,
                "avg_low_price": 100500,
                "low_price_volume": 170
            }
        ]);

        let output = super::simulate_order_from_json(
            &serde_json::to_string(&request_payload).unwrap(),
            &serde_json::to_string(&history_payload).unwrap(),
        )
        .unwrap();
        let parsed: PaperBetOutcome = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed.status, SimulatedOrderStatus::Closed);
        assert_eq!(parsed.entry_time.to_rfc3339(), "2026-06-16T14:00:00+00:00");
    }
}
