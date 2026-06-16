use std::{
    fs,
    path::{Path, PathBuf},
};

use chrono::{TimeZone, Utc};
use grand_edge_domain::{FeatureVector, ItemId};
use grand_edge_model_runtime::{InferenceRequest, ModelRuntime, ModelRuntimeError};
use serde_json::{Map, json};
use tempfile::tempdir;

fn fixture_root(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn copy_fixture_dir(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_fixture_dir(&source_path, &destination_path)?;
        } else {
            fs::copy(&source_path, &destination_path)?;
        }
    }
    Ok(())
}

fn python_export_bundle() -> PathBuf {
    fixture_root("python_export")
        .join("gbm_ranker_v1")
        .join("2026-06-16.1")
}

fn coefficient_bundle() -> PathBuf {
    fixture_root("coefficients")
        .join("meta_label_v1")
        .join("2026-06-16.1")
}

fn runtime() -> ModelRuntime {
    ModelRuntime::new(fixture_root("python_export"))
}

fn validation_as_of() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap()
}

fn feature_vector_for_names(feature_names: &[&str]) -> FeatureVector {
    let mut values = Map::new();
    for name in feature_names {
        let value = match *name {
            "missing_feature_policy" => json!("null_when_inputs_missing"),
            "missing_data_flags" => json!([]),
            _ => json!(1.0),
        };
        values.insert((*name).to_string(), value);
    }

    FeatureVector {
        item_id: ItemId(4151),
        as_of: validation_as_of(),
        feature_set_version: "features_v1".to_string(),
        values,
    }
}

fn coefficient_feature_vector() -> FeatureVector {
    let mut values = Map::new();
    values.insert("return_1h".to_string(), json!(1.0));
    values.insert("return_6h".to_string(), json!(1.0));
    values.insert("observed_volume_z_24h".to_string(), json!(1.0));
    values.insert("spread_pct".to_string(), json!(0.7181818181818181));

    FeatureVector {
        item_id: ItemId(4151),
        as_of: validation_as_of(),
        feature_set_version: "features_v1".to_string(),
        values,
    }
}

#[test]
fn validate_bundle_path_accepts_fixture_export() {
    let bundle = runtime()
        .validate_bundle_path(&python_export_bundle(), validation_as_of())
        .unwrap();

    assert_eq!(bundle.bundle.metadata.strategy_id, "gbm_ranker_v1");
    assert_eq!(
        bundle.bundle.metadata.feature_schema_hash,
        "sha256:05268d9b1b7bd88ba6c5cf09f5d7423011a0015478f587e905c0cef83e1386be"
    );
}

#[test]
fn rejects_missing_model_card() {
    let temp = tempdir().unwrap();
    let root = temp.path().join("bundle");
    copy_fixture_dir(&python_export_bundle(), &root).unwrap();
    fs::remove_file(root.join("model_card.json")).unwrap();

    let error = runtime()
        .validate_bundle_path(&root, validation_as_of())
        .unwrap_err();
    assert!(matches!(error, ModelRuntimeError::MissingFile(_)));
}

#[test]
fn rejects_feature_schema_hash_mismatch() {
    let temp = tempdir().unwrap();
    let root = temp.path().join("bundle");
    copy_fixture_dir(&python_export_bundle(), &root).unwrap();
    let schema_path = root.join("feature_schema.json");
    let mutated = fs::read_to_string(&schema_path).unwrap().replace(
        "05268d9b1b7bd88ba6c5cf09f5d7423011a0015478f587e905c0cef83e1386be",
        "deadbeef",
    );
    fs::write(schema_path, mutated).unwrap();

    let error = runtime()
        .validate_bundle_path(&root, validation_as_of())
        .unwrap_err();
    assert!(matches!(
        error,
        ModelRuntimeError::FeatureSchemaHashMismatch
    ));
}

#[test]
fn rejects_training_window_after_as_of() {
    let temp = tempdir().unwrap();
    let root = temp.path().join("bundle");
    copy_fixture_dir(&python_export_bundle(), &root).unwrap();
    let model_card_path = root.join("model_card.json");
    let mutated = fs::read_to_string(&model_card_path).unwrap().replace(
        "\"training_window_end\": \"2026-05-01T00:00:00Z\"",
        "\"training_window_end\": \"2026-06-20T00:00:00Z\"",
    );
    fs::write(model_card_path, mutated).unwrap();

    let error = runtime()
        .validate_bundle_path(&root, validation_as_of())
        .unwrap_err();
    assert!(
        matches!(error, ModelRuntimeError::Validation(message) if message.contains("training window"))
    );
}

#[test]
fn coefficient_model_scores_fixture() {
    let runtime = ModelRuntime::new(fixture_root("coefficients"));
    let artifact = runtime
        .validate_bundle_path(&coefficient_bundle(), validation_as_of())
        .unwrap();
    let request = InferenceRequest {
        feature_snapshot_id: uuid::Uuid::new_v4(),
        item_id: ItemId(4151),
        as_of: validation_as_of(),
        feature_vector: coefficient_feature_vector(),
        artifact,
    };

    let output = runtime.infer(request).unwrap();
    assert!((output.prediction.predicted_return.unwrap().get() - 0.3).abs() < 1e-12);
    assert!((output.prediction.confidence.get() - 0.574_442_516_8).abs() < 1e-9);
    assert_eq!(output.prediction.explanation["backend"], "coefficients");
    assert!(output.prediction.explanation.get("action").is_none());
}

#[test]
fn onnx_artifact_requires_feature_flag() {
    let runtime = runtime();
    let artifact = runtime
        .validate_bundle_path(&python_export_bundle(), validation_as_of())
        .unwrap();
    let request = InferenceRequest {
        feature_snapshot_id: uuid::Uuid::new_v4(),
        item_id: ItemId(4151),
        as_of: validation_as_of(),
        feature_vector: feature_vector_for_names(&[
            "mid",
            "spread_abs",
            "spread_pct",
            "return_5m",
            "return_1h",
            "return_6h",
            "return_24h",
            "rolling_mean_24h",
            "rolling_std_24h",
            "z_score_24h",
            "ewma_volatility_24h",
            "observed_volume_1h",
            "observed_high_side_volume_1h",
            "observed_low_side_volume_1h",
            "observed_volume_z_24h",
            "observed_volume_reliability_24h",
            "high_low_volume_ratio_1h",
            "spread_stability_24h",
            "price_staleness_secs",
            "alch_floor_distance",
            "buy_limit",
            "buy_limit_utilization",
            "liquidity_confidence",
            "missing_feature_policy",
            "missing_data_flags",
        ]),
        artifact,
    };

    let error = runtime.infer(request).unwrap_err();
    assert!(matches!(
        error,
        ModelRuntimeError::UnsupportedArtifactKind("gbdt_ranker")
    ));
}
