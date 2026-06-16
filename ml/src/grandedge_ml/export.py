from __future__ import annotations

import argparse
import json
from dataclasses import asdict
from datetime import UTC, datetime
from pathlib import Path

from .artifact_schema import (
    ArtifactBundle,
    CalibrationMetadata,
    GraphArtifactMetadata,
    ModelArtifactKind,
    ModelArtifactMetadata,
    ModelCard,
    TrainingTargetLabel,
    validate_artifact_metadata,
)
from .calibration import build_calibration_metadata
from .features import (
    FEATURE_SET_VERSION,
    default_feature_schema,
    feature_schema_hash,
    graph_feature_schema,
    GRAPH_FEATURE_SET_VERSION,
)


def isoformat_z(value: datetime) -> str:
    return value.astimezone(UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def default_fixture_timestamps() -> tuple[datetime, datetime, datetime, datetime, datetime]:
    return (
        datetime(2026, 6, 16, 12, 0, tzinfo=UTC),
        datetime(2026, 1, 1, 0, 0, tzinfo=UTC),
        datetime(2026, 5, 1, 0, 0, tzinfo=UTC),
        datetime(2026, 5, 1, 0, 0, tzinfo=UTC),
        datetime(2026, 6, 1, 0, 0, tzinfo=UTC),
    )


def _serialize_calibration(metadata: CalibrationMetadata) -> dict[str, object]:
    payload = asdict(metadata)
    payload["fitted_at"] = isoformat_z(metadata.fitted_at)
    return payload


def _graph_metadata(
    *,
    training_start: datetime,
    evaluation_end: datetime,
    schema,
) -> GraphArtifactMetadata:
    return GraphArtifactMetadata(
        graph_feature_set_version=GRAPH_FEATURE_SET_VERSION,
        graph_version="graph_v1",
        relation_corpus_hash="sha256:graph-corpus-fixture",
        edge_observation_window_start=training_start,
        edge_observation_window_end=evaluation_end,
        graph_feature_groups=schema.graph_feature_groups,
    )


def _artifact_kind_for_strategy(strategy_id: str) -> ModelArtifactKind:
    if strategy_id.startswith("graph_gnn") or strategy_id.startswith("graph_neural_network"):
        return ModelArtifactKind.GRAPH_NEURAL_NETWORK_DEFERRED
    if strategy_id.startswith("graph_ranker"):
        return ModelArtifactKind.GRAPH_RANKER
    return ModelArtifactKind.GBDT_RANKER


def write_fixture_bundle(
    *,
    output_root: Path,
    strategy_id: str = "gbm_ranker_v1",
    model_version: str = "2026-06-16.1",
    target_label: TrainingTargetLabel = TrainingTargetLabel.FUTURE_ACTIONABLE_RETURN_6H,
) -> ArtifactBundle:
    as_of, training_start, training_end, evaluation_start, evaluation_end = (
        default_fixture_timestamps()
    )
    artifact_kind = _artifact_kind_for_strategy(strategy_id)
    is_graph = artifact_kind in {
        ModelArtifactKind.GRAPH_RANKER,
        ModelArtifactKind.GRAPH_NEURAL_NETWORK_DEFERRED,
    }
    feature_set_version = GRAPH_FEATURE_SET_VERSION if is_graph else FEATURE_SET_VERSION
    schema = graph_feature_schema(target_label) if is_graph else default_feature_schema(target_label)
    schema_hash = feature_schema_hash(schema, feature_set_version)
    bundle_root = output_root / strategy_id / model_version
    bundle_root.mkdir(parents=True, exist_ok=True)
    graph = _graph_metadata(
        training_start=training_start,
        evaluation_end=evaluation_end,
        schema=schema,
    ) if is_graph else None

    metadata = ModelArtifactMetadata(
        strategy_id=strategy_id,
        model_version=model_version,
        feature_set_version=feature_set_version,
        feature_schema_hash=schema_hash,
        trained_at=as_of,
        training_window_start=training_start,
        training_window_end=training_end,
        evaluation_window_start=evaluation_start,
        evaluation_window_end=evaluation_end,
        artifact_uri=(bundle_root / "model.onnx").resolve().as_uri(),
        artifact_kind=artifact_kind,
        graph=graph,
    )
    validate_artifact_metadata(
        metadata,
        expected_strategy_id=strategy_id,
        expected_feature_set_version=feature_set_version,
        as_of=as_of,
    )

    card = ModelCard(
        strategy_id=strategy_id,
        model_version=model_version,
        feature_set_version=feature_set_version,
        feature_schema_hash=schema_hash,
        training_window_start=training_start,
        training_window_end=training_end,
        evaluation_window_start=evaluation_start,
        evaluation_window_end=evaluation_end,
        metrics={"directional_accuracy": 0.61, "window": "seven_days"},
        known_limitations=[
            "Observed volume and liquidity confidence are proxy inputs, not true order-book depth."
        ],
        target_label=target_label,
        notes="Fixture export only. Rust remains the production validation and serving boundary.",
        graph=graph,
    )
    calibration = build_calibration_metadata(
        bins=[{"predicted": 0.60, "realized": 0.58}],
        fitted_at=as_of,
    )

    feature_schema_path = bundle_root / "feature_schema.json"
    model_card_path = bundle_root / "model_card.json"
    calibration_path = bundle_root / "calibration.json"
    model_path = bundle_root / "model.onnx"

    feature_schema_path.write_text(
        json.dumps(
            {
                "feature_set_version": feature_set_version,
                "feature_names": schema.feature_names,
                "target_label": schema.target_label.value,
                "graph_feature_groups": [group.value for group in schema.graph_feature_groups],
                "feature_schema_hash": schema_hash,
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )
    model_card_path.write_text(
        json.dumps(
            {
                "strategy_id": card.strategy_id,
                "model_version": card.model_version,
                "feature_set_version": card.feature_set_version,
                "feature_schema_hash": card.feature_schema_hash,
                "training_window_start": isoformat_z(card.training_window_start),
                "training_window_end": isoformat_z(card.training_window_end),
                "evaluation_window_start": isoformat_z(card.evaluation_window_start),
                "evaluation_window_end": isoformat_z(card.evaluation_window_end),
                "metrics": card.metrics,
                "known_limitations": card.known_limitations,
                "target_label": card.target_label.value,
                "notes": card.notes,
                "graph": None
                if card.graph is None
                else {
                    "graph_feature_set_version": card.graph.graph_feature_set_version,
                    "graph_version": card.graph.graph_version,
                    "relation_corpus_hash": card.graph.relation_corpus_hash,
                    "edge_observation_window_start": isoformat_z(
                        card.graph.edge_observation_window_start
                    ),
                    "edge_observation_window_end": isoformat_z(
                        card.graph.edge_observation_window_end
                    ),
                    "graph_feature_groups": [
                        group.value for group in card.graph.graph_feature_groups
                    ],
                },
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )
    calibration_path.write_text(
        json.dumps(_serialize_calibration(calibration), indent=2) + "\n",
        encoding="utf-8",
    )
    model_path.write_bytes(b"fixture-onnx-placeholder\n")

    return ArtifactBundle(
        strategy_id=strategy_id,
        model_version=model_version,
        feature_set_version=feature_set_version,
        feature_schema_hash=schema_hash,
        training_window_start=training_start,
        training_window_end=training_end,
        evaluation_window_start=evaluation_start,
        evaluation_window_end=evaluation_end,
        files={
            "model_card": model_card_path,
            "feature_schema": feature_schema_path,
            "calibration": calibration_path,
            "model": model_path,
        },
    )


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="python -m grandedge_ml.export",
        description=(
            "Export research-only Grand Edge artifact bundles for Rust validation. "
            "Bundle files: model.onnx, model_card.json, feature_schema.json, calibration.json."
        ),
    )
    parser.add_argument(
        "--output-root",
        type=Path,
        default=Path("ml/artifacts"),
        help=(
            "Artifact root directory. Bundle output is "
            "{output_root}/{strategy_id}/{model_version}/."
        ),
    )
    parser.add_argument("--strategy-id", default="gbm_ranker_v1")
    parser.add_argument("--model-version", default="2026-06-16.1")
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    bundle = write_fixture_bundle(
        output_root=args.output_root,
        strategy_id=args.strategy_id,
        model_version=args.model_version,
    )
    print(bundle.files["model_card"].parent)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
