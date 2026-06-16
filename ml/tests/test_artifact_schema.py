from datetime import UTC, datetime

import pytest

from grandedge_ml.artifact_schema import (
    ModelArtifactKind,
    ModelArtifactMetadata,
    validate_artifact_metadata,
)


def build_metadata() -> ModelArtifactMetadata:
    return ModelArtifactMetadata(
        strategy_id="gbm_ranker_v1",
        model_version="2026-06-16.1",
        feature_set_version="features_v1",
        feature_schema_hash="sha256:abc",
        trained_at=datetime(2026, 6, 16, 12, 0, tzinfo=UTC),
        training_window_start=datetime(2026, 1, 1, 0, 0, tzinfo=UTC),
        training_window_end=datetime(2026, 5, 1, 0, 0, tzinfo=UTC),
        evaluation_window_start=datetime(2026, 5, 1, 0, 0, tzinfo=UTC),
        evaluation_window_end=datetime(2026, 6, 1, 0, 0, tzinfo=UTC),
        artifact_uri="file:///tmp/model.onnx",
        artifact_kind=ModelArtifactKind.GBDT_RANKER,
    )


def test_model_card_requires_feature_schema_hash() -> None:
    metadata = build_metadata()
    metadata = ModelArtifactMetadata(**{**metadata.__dict__, "feature_schema_hash": ""})

    with pytest.raises(ValueError, match="feature_schema_hash"):
        validate_artifact_metadata(
            metadata,
            expected_strategy_id="gbm_ranker_v1",
            expected_feature_set_version="features_v1",
            as_of=datetime(2026, 6, 16, 12, 0, tzinfo=UTC),
        )


def test_training_window_must_end_before_evaluation_window() -> None:
    metadata = build_metadata()
    metadata = ModelArtifactMetadata(
        **{
            **metadata.__dict__,
            "evaluation_window_start": datetime(2026, 4, 30, 23, 0, tzinfo=UTC),
        }
    )

    with pytest.raises(ValueError, match="evaluation window must start after training window end"):
        validate_artifact_metadata(
            metadata,
            expected_strategy_id="gbm_ranker_v1",
            expected_feature_set_version="features_v1",
            as_of=datetime(2026, 6, 16, 12, 0, tzinfo=UTC),
        )


def test_artifact_rejects_future_training_window() -> None:
    metadata = build_metadata()
    metadata = ModelArtifactMetadata(
        **{
            **metadata.__dict__,
            "training_window_end": datetime(2026, 6, 17, 0, 0, tzinfo=UTC),
        }
    )

    with pytest.raises(ValueError, match="must not extend past as_of"):
        validate_artifact_metadata(
            metadata,
            expected_strategy_id="gbm_ranker_v1",
            expected_feature_set_version="features_v1",
            as_of=datetime(2026, 6, 16, 12, 0, tzinfo=UTC),
        )
