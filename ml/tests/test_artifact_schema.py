from datetime import UTC, datetime
import json
from pathlib import Path

import jsonschema
import pytest

from grandedge_ml.artifact_schema import (
    GraphArtifactMetadata,
    GraphFeatureGroup,
    ModelArtifactKind,
    ModelArtifactMetadata,
    load_rust_schema,
    validate_against_rust_schema,
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


def test_model_artifact_metadata_matches_rust_schema() -> None:
    metadata = build_metadata()
    validate_against_rust_schema(metadata.__dict__, "model_artifact_metadata")


def test_graph_artifact_requires_graph_version() -> None:
    metadata = ModelArtifactMetadata(
        **{
            **build_metadata().__dict__,
            "strategy_id": "graph_ranker_v1",
            "feature_set_version": "graph_features_v1",
            "artifact_kind": ModelArtifactKind.GRAPH_RANKER,
            "graph": GraphArtifactMetadata(
                graph_feature_set_version="graph_features_v1",
                graph_version="",
                relation_corpus_hash="sha256:graph-corpus-fixture",
                edge_observation_window_start=datetime(2026, 1, 1, 0, 0, tzinfo=UTC),
                edge_observation_window_end=datetime(2026, 6, 1, 0, 0, tzinfo=UTC),
                graph_feature_groups=[GraphFeatureGroup.NEIGHBOR_RETURN_FEATURES],
            ),
        }
    )

    with pytest.raises(ValueError, match="graph_version"):
        validate_artifact_metadata(
            metadata,
            expected_strategy_id="graph_ranker_v1",
            expected_feature_set_version="graph_features_v1",
            as_of=datetime(2026, 6, 16, 12, 0, tzinfo=UTC),
        )


def test_model_card_fixture_matches_rust_schema() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    fixture = (
        repo_root
        / "crates"
        / "model_runtime"
        / "tests"
        / "fixtures"
        / "python_export"
        / "gbm_ranker_v1"
        / "2026-06-16.1"
        / "model_card.json"
    )
    payload = json.loads(fixture.read_text(encoding="utf-8"))
    schema = load_rust_schema("model_card_document")
    jsonschema.Draft202012Validator(schema).validate(payload)
