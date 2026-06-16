import pytest

from grandedge_ml.artifact_schema import ArtifactFeatureSchema, TrainingTargetLabel
from grandedge_ml.features import (
    FEATURE_SET_VERSION,
    default_feature_schema,
    feature_schema_hash,
    graph_feature_schema,
    validate_feature_names,
)


def test_python_feature_schema_matches_fixture_version() -> None:
    schema = default_feature_schema()
    assert FEATURE_SET_VERSION == "features_v1"
    assert schema.target_label == TrainingTargetLabel.FUTURE_ACTIONABLE_RETURN_6H
    assert feature_schema_hash(schema, FEATURE_SET_VERSION).startswith("sha256:")


def test_feature_schema_rejects_true_liquidity_names() -> None:
    schema = ArtifactFeatureSchema(
        feature_names=["observed_volume_z_24h", "trueLiquidity"],
        target_label=TrainingTargetLabel.FUTURE_ACTIONABLE_RETURN_6H,
        graph_feature_groups=[],
    )

    with pytest.raises(ValueError, match="forbidden true-liquidity names"):
        validate_feature_names(schema.feature_names)


def test_graph_feature_groups_include_path_features() -> None:
    schema = graph_feature_schema()

    assert "strongest_graph_path_confidence" in schema.feature_names
    assert any(group.value == "neighbor_return_features" for group in schema.graph_feature_groups)
