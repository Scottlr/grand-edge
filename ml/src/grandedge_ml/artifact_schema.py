from __future__ import annotations

from dataclasses import asdict, dataclass, is_dataclass
import json
from datetime import UTC, datetime
from enum import StrEnum
from pathlib import Path

import jsonschema


class ModelArtifactKind(StrEnum):
    GBDT_RANKER = "gbdt_ranker"
    GRAPH_RANKER = "graph_ranker"
    GRAPH_NEURAL_NETWORK_DEFERRED = "graph_neural_network_deferred"
    CONTEXTUAL_BANDIT = "contextual_bandit"
    ONLINE_ENSEMBLE = "online_ensemble"
    META_LABEL = "meta_label"


class TrainingTargetLabel(StrEnum):
    FUTURE_RETURN_6H = "future_return_6h"
    FUTURE_TAX_ADJUSTED_RETURN_6H = "future_tax_adjusted_return_6h"
    FUTURE_ACTIONABLE_RETURN_6H = "future_actionable_return_6h"


class GraphFeatureGroup(StrEnum):
    OWN_ITEM_FEATURES = "own_item_features"
    OBSERVED_EXECUTION_PROXY_FEATURES = "observed_execution_proxy_features"
    NEIGHBOR_RETURN_FEATURES = "neighbor_return_features"
    SECTOR_FEATURES = "sector_features"
    CONVERSION_FEATURES = "conversion_features"
    SHOCK_FEATURES = "shock_features"
    EDGE_STABILITY_FEATURES = "edge_stability_features"
    EVENT_FEATURES = "event_features"
    MISSING_DATA_FLAGS = "missing_data_flags"


@dataclass(frozen=True)
class GraphArtifactMetadata:
    graph_feature_set_version: str
    graph_version: str
    relation_corpus_hash: str
    edge_observation_window_start: datetime
    edge_observation_window_end: datetime
    graph_feature_groups: list[GraphFeatureGroup]


@dataclass(frozen=True)
class ArtifactFeatureSchema:
    feature_names: list[str]
    target_label: TrainingTargetLabel
    graph_feature_groups: list[GraphFeatureGroup]


@dataclass(frozen=True)
class CalibrationMetadata:
    method: str
    fitted_at: datetime
    bins: list[dict[str, float]]


@dataclass(frozen=True)
class ModelCard:
    strategy_id: str
    model_version: str
    feature_set_version: str
    feature_schema_hash: str
    training_window_start: datetime
    training_window_end: datetime
    evaluation_window_start: datetime
    evaluation_window_end: datetime
    metrics: dict[str, float | int | str | None]
    known_limitations: list[str]
    target_label: TrainingTargetLabel
    notes: str
    graph: GraphArtifactMetadata | None = None


@dataclass(frozen=True)
class ModelArtifactMetadata:
    strategy_id: str
    model_version: str
    feature_set_version: str
    feature_schema_hash: str
    trained_at: datetime
    training_window_start: datetime
    training_window_end: datetime
    evaluation_window_start: datetime
    evaluation_window_end: datetime
    artifact_uri: str
    artifact_kind: ModelArtifactKind
    graph: GraphArtifactMetadata | None = None


@dataclass(frozen=True)
class ArtifactBundle:
    strategy_id: str
    model_version: str
    feature_set_version: str
    feature_schema_hash: str
    training_window_start: datetime
    training_window_end: datetime
    evaluation_window_start: datetime
    evaluation_window_end: datetime
    files: dict[str, Path]


def ensure_utc(timestamp: datetime) -> datetime:
    if timestamp.tzinfo is None:
        return timestamp.replace(tzinfo=UTC)
    return timestamp.astimezone(UTC)


def validate_artifact_metadata(
    metadata: ModelArtifactMetadata,
    *,
    expected_strategy_id: str,
    expected_feature_set_version: str,
    as_of: datetime,
) -> None:
    as_of = ensure_utc(as_of)
    trained_at = ensure_utc(metadata.trained_at)
    training_window_start = ensure_utc(metadata.training_window_start)
    training_window_end = ensure_utc(metadata.training_window_end)
    evaluation_window_start = ensure_utc(metadata.evaluation_window_start)
    evaluation_window_end = ensure_utc(metadata.evaluation_window_end)

    if metadata.strategy_id != expected_strategy_id:
        raise ValueError("artifact strategy_id did not match expected strategy")
    if metadata.feature_set_version != expected_feature_set_version:
        raise ValueError("artifact feature_set_version did not match expected version")
    if not metadata.model_version.strip():
        raise ValueError("artifact model_version must not be empty")
    if not metadata.feature_schema_hash.strip():
        raise ValueError("model card requires feature_schema_hash")
    if not metadata.artifact_uri.strip():
        raise ValueError("artifact uri must not be empty")
    if training_window_end > as_of:
        raise ValueError("artifact training window must not extend past as_of")
    if training_window_end < training_window_start:
        raise ValueError("artifact training window end must be after start")
    if evaluation_window_start < training_window_end:
        raise ValueError("artifact evaluation window must start after training window end")
    if evaluation_window_end < evaluation_window_start:
        raise ValueError("artifact evaluation window end must be after evaluation start")
    if evaluation_window_end > as_of:
        raise ValueError("artifact evaluation window must not extend past as_of")
    if trained_at < training_window_start:
        raise ValueError("trained_at must be after the training window start")
    if metadata.artifact_kind in {
        ModelArtifactKind.GRAPH_RANKER,
        ModelArtifactKind.GRAPH_NEURAL_NETWORK_DEFERRED,
    }:
        if metadata.graph is None:
            raise ValueError("graph artifacts require graph metadata")
        if not metadata.graph.graph_feature_set_version.strip():
            raise ValueError("graph artifacts require graph_feature_set_version")
        if not metadata.graph.graph_version.strip():
            raise ValueError("graph artifacts require graph_version")
        if not metadata.graph.relation_corpus_hash.strip():
            raise ValueError("graph artifacts require relation_corpus_hash")
        if not metadata.graph.graph_feature_groups:
            raise ValueError("graph artifacts require graph_feature_groups")


def load_rust_schema(schema_name: str) -> dict:
    schema_path = (
        Path(__file__).resolve().parents[3]
        / "schemas"
        / f"{schema_name}.schema.json"
    )
    return json.loads(schema_path.read_text(encoding="utf-8"))


def validate_against_rust_schema(payload: dict, schema_name: str) -> None:
    schema = load_rust_schema(schema_name)
    jsonschema.Draft202012Validator(schema).validate(to_jsonable(payload))


def to_jsonable(value):
    if is_dataclass(value):
        return to_jsonable(asdict(value))
    if isinstance(value, datetime):
        return ensure_utc(value).isoformat().replace("+00:00", "Z")
    if isinstance(value, StrEnum):
        return str(value)
    if isinstance(value, dict):
        return {key: to_jsonable(item) for key, item in value.items()}
    if isinstance(value, list):
        return [to_jsonable(item) for item in value]
    return value
