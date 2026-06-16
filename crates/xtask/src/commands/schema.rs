use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use grand_edge_api::app::openapi_document;
use grand_edge_domain::{RecommendationExplanation, StructuredRecommendationExplanation};
use grand_edge_model_runtime::{ArtifactFeatureSchemaDocument, ModelCardDocument};
use grand_edge_strategies::{ModelArtifactMetadata, RiskConfig, StrategyConfig};
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportedSchema {
    pub name: String,
    pub path: PathBuf,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaExportManifest {
    pub generator: String,
    pub generated_at: DateTime<Utc>,
    pub schemas: Vec<ExportedSchema>,
}

#[derive(Debug, thiserror::Error)]
pub enum SchemaExportError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub async fn schema_export(out: &str) -> Result<String, SchemaExportError> {
    let output_dir = resolve_output_dir(out)?;
    let manifest = export_schemas(&output_dir)?;
    Ok(serde_json::to_string_pretty(&manifest)?)
}

pub fn export_schemas(output_dir: &Path) -> Result<SchemaExportManifest, SchemaExportError> {
    fs::create_dir_all(output_dir)?;

    let mut exports = vec![
        schema_entry(
            "artifact_feature_schema_document",
            "artifact_feature_schema_document.schema.json",
            schema_value::<ArtifactFeatureSchemaDocument>()?,
        ),
        schema_entry(
            "model_artifact_metadata",
            "model_artifact_metadata.schema.json",
            schema_value::<ModelArtifactMetadata>()?,
        ),
        schema_entry(
            "model_card_document",
            "model_card_document.schema.json",
            schema_value::<ModelCardDocument>()?,
        ),
        schema_entry(
            "openapi",
            "openapi.json",
            canonical_json_value(serde_json::to_value(openapi_document())?),
        ),
        schema_entry(
            "recommendation_explanation",
            "recommendation_explanation.schema.json",
            schema_value::<RecommendationExplanation>()?,
        ),
        schema_entry(
            "risk_config",
            "risk_config.schema.json",
            schema_value::<RiskConfig>()?,
        ),
        schema_entry(
            "strategy_config",
            "strategy_config.schema.json",
            schema_value::<StrategyConfig>()?,
        ),
        schema_entry(
            "structured_recommendation_explanation",
            "structured_recommendation_explanation.schema.json",
            schema_value::<StructuredRecommendationExplanation>()?,
        ),
    ];
    exports.sort_by(|left, right| left.name.cmp(&right.name));

    let mut manifest_entries = Vec::with_capacity(exports.len());
    for export in exports {
        let path = output_dir.join(&export.file_name);
        let bytes = serde_json::to_vec_pretty(&export.contents)?;
        fs::write(&path, bytes)?;
        manifest_entries.push(ExportedSchema {
            name: export.name,
            path: PathBuf::from(&export.file_name),
            sha256: sha256_hex(&fs::read(&path)?),
        });
    }

    let manifest = SchemaExportManifest {
        generator: format!("grand-edge-xtask@{}", env!("CARGO_PKG_VERSION")),
        generated_at: Utc::now(),
        schemas: manifest_entries,
    };
    let manifest_path = output_dir.join("schema-manifest.json");
    fs::write(
        manifest_path,
        serde_json::to_vec_pretty(&canonical_json_value(serde_json::to_value(&manifest)?))?,
    )?;

    Ok(manifest)
}

fn resolve_output_dir(out: &str) -> Result<PathBuf, std::io::Error> {
    let candidate = Path::new(out);
    if candidate.is_absolute() {
        return Ok(candidate.to_path_buf());
    }

    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(candidate))
}

fn schema_value<T>() -> Result<Value, serde_json::Error>
where
    T: JsonSchema,
{
    serde_json::to_value(schema_for!(T)).map(canonical_json_value)
}

fn canonical_json_value(value: Value) -> Value {
    match value {
        Value::Array(values) => {
            Value::Array(values.into_iter().map(canonical_json_value).collect())
        }
        Value::Object(map) => {
            let sorted = map
                .into_iter()
                .map(|(key, value)| (key, canonical_json_value(value)))
                .collect();
            Value::Object(sorted)
        }
        scalar => scalar,
    }
}

fn schema_entry(name: &str, file_name: &str, contents: Value) -> PendingSchema {
    PendingSchema {
        name: name.to_string(),
        file_name: file_name.to_string(),
        contents,
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("{digest:x}")
}

struct PendingSchema {
    name: String,
    file_name: String,
    contents: Value,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{export_schemas, schema_value};
    use grand_edge_domain::RecommendationExplanation;
    use grand_edge_strategies::{ModelArtifactMetadata, StrategyConfig};

    fn temp_output_dir() -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("grandedge-schema-export-{unique}"))
    }

    #[test]
    fn schema_export_writes_expected_files() {
        let output_dir = temp_output_dir();
        export_schemas(&output_dir).unwrap();

        let expected = BTreeSet::from([
            "artifact_feature_schema_document.schema.json".to_string(),
            "model_artifact_metadata.schema.json".to_string(),
            "model_card_document.schema.json".to_string(),
            "openapi.json".to_string(),
            "recommendation_explanation.schema.json".to_string(),
            "risk_config.schema.json".to_string(),
            "schema-manifest.json".to_string(),
            "strategy_config.schema.json".to_string(),
            "structured_recommendation_explanation.schema.json".to_string(),
        ]);

        let actual = std::fs::read_dir(&output_dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect::<BTreeSet<_>>();
        assert_eq!(actual, expected);
        assert!(output_dir.join("schema-manifest.json").is_file());
    }

    #[test]
    fn schema_manifest_uses_relative_paths_and_hashes() {
        let output_dir = temp_output_dir();
        let manifest = export_schemas(&output_dir).unwrap();

        for schema in manifest.schemas {
            assert!(!schema.path.is_absolute());
            assert_eq!(schema.sha256.len(), 64);
            assert!(schema.sha256.chars().all(|ch| ch.is_ascii_hexdigit()));
        }
    }

    #[test]
    fn strategy_config_schema_snapshot() {
        insta::assert_json_snapshot!(
            "strategy_config_schema",
            schema_value::<StrategyConfig>().unwrap()
        );
    }

    #[test]
    fn model_artifact_schema_snapshot() {
        insta::assert_json_snapshot!(
            "model_artifact_metadata_schema",
            schema_value::<ModelArtifactMetadata>().unwrap()
        );
    }

    #[test]
    fn recommendation_explanation_schema_snapshot() {
        insta::assert_json_snapshot!(
            "recommendation_explanation_schema",
            schema_value::<RecommendationExplanation>().unwrap()
        );
    }
}
