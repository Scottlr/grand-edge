use grand_edge_domain::{FeatureVector, Probability, Rate};
use serde_json::{Map, Value, json};

use crate::{
    artifacts::ValidatedArtifactBundle,
    errors::ModelRuntimeError,
    runtime::{InferenceOutput, InferenceRequest},
};

pub fn infer(
    request: InferenceRequest,
    bundle: &ValidatedArtifactBundle,
) -> Result<InferenceOutput, ModelRuntimeError> {
    let coefficient_model = bundle
        .coefficient_model
        .as_ref()
        .ok_or_else(|| ModelRuntimeError::Validation("missing coefficient model".to_string()))?;

    let mut linear_score = coefficient_model.intercept;
    let mut contributions = Map::new();
    for (feature_name, weight) in coefficient_model
        .features
        .iter()
        .zip(coefficient_model.weights.iter())
    {
        let value = feature_number(&request.feature_vector, feature_name)?;
        let contribution = value * weight;
        linear_score += contribution;
        contributions.insert(
            feature_name.clone(),
            json!({
                "value": value,
                "weight": weight,
                "contribution": contribution,
            }),
        );
    }

    let probability_positive = sigmoid(linear_score);
    Ok(InferenceOutput {
        strategy_id: grand_edge_domain::StrategyId::new(
            bundle.bundle.metadata.strategy_id.clone(),
        )?,
        model_version: grand_edge_domain::ModelVersion::new(
            bundle.bundle.metadata.model_version.clone(),
        )?,
        item_id: request.item_id,
        as_of: request.as_of,
        expected_return: Rate::new(linear_score)?,
        probability_positive: Probability::new(probability_positive)?,
        explanation: json!({
            "backend": "coefficients",
            "model_id": coefficient_model.model_id,
            "feature_schema_hash": bundle.feature_schema_hash(),
            "target_label": bundle.target_label(),
            "linear_score": linear_score,
            "contributions": contributions,
        }),
    })
}

fn feature_number(feature_vector: &FeatureVector, name: &str) -> Result<f64, ModelRuntimeError> {
    feature_vector
        .values
        .get(name)
        .and_then(Value::as_f64)
        .ok_or_else(|| {
            ModelRuntimeError::FeatureSchemaMismatch(format!(
                "feature `{name}` missing or non-numeric for coefficient inference"
            ))
        })
}

fn sigmoid(value: f64) -> f64 {
    1.0 / (1.0 + (-value).exp())
}
