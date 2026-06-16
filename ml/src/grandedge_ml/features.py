from __future__ import annotations

import hashlib
from dataclasses import dataclass

from .artifact_schema import (
    ArtifactFeatureSchema,
    GraphFeatureGroup,
    TrainingTargetLabel,
)

FEATURE_SET_VERSION = "features_v1"
GRAPH_FEATURE_SET_VERSION = "graph_features_v1"
FORBIDDEN_FEATURE_NAMES = {
    "trueLiquidity",
    "marketDepth",
    "availableQuantity",
    "orderBookDepth",
    "exactExecutableQuantity",
}

CORE_FEATURE_NAMES = [
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
]

GRAPH_FEATURE_NAMES = [
    "upstream_pressure_1h",
    "downstream_pressure_1h",
    "relative_value_residual",
    "conversion_gap_pct",
    "graph_adjusted_momentum_6h",
    "link_disagreement_6h",
    "strongest_graph_path_confidence",
    "graph_neighbor_count",
    "graph_missing_neighbor_data_count",
    "edge_stability_score",
    "event_exposure_score",
]

GRAPH_FEATURE_GROUPS = [
    GraphFeatureGroup.OWN_ITEM_FEATURES,
    GraphFeatureGroup.OBSERVED_EXECUTION_PROXY_FEATURES,
    GraphFeatureGroup.NEIGHBOR_RETURN_FEATURES,
    GraphFeatureGroup.SECTOR_FEATURES,
    GraphFeatureGroup.CONVERSION_FEATURES,
    GraphFeatureGroup.SHOCK_FEATURES,
    GraphFeatureGroup.EDGE_STABILITY_FEATURES,
    GraphFeatureGroup.EVENT_FEATURES,
    GraphFeatureGroup.MISSING_DATA_FLAGS,
]


@dataclass(frozen=True)
class FeatureSchemaDocument:
    feature_set_version: str
    feature_names: list[str]
    target_label: str
    graph_feature_groups: list[str]


def validate_feature_names(feature_names: list[str]) -> list[str]:
    blocked = sorted(name for name in feature_names if name in FORBIDDEN_FEATURE_NAMES)
    if blocked:
        raise ValueError(f"feature schema contains forbidden true-liquidity names: {blocked}")
    return feature_names


def default_feature_schema(
    target_label: TrainingTargetLabel = TrainingTargetLabel.FUTURE_ACTIONABLE_RETURN_6H,
) -> ArtifactFeatureSchema:
    return ArtifactFeatureSchema(
        feature_names=validate_feature_names(CORE_FEATURE_NAMES.copy()),
        target_label=target_label,
        graph_feature_groups=[],
    )


def graph_feature_schema(
    target_label: TrainingTargetLabel = TrainingTargetLabel.FUTURE_ACTIONABLE_RETURN_6H,
) -> ArtifactFeatureSchema:
    feature_names = validate_feature_names(CORE_FEATURE_NAMES.copy() + GRAPH_FEATURE_NAMES.copy())
    return ArtifactFeatureSchema(
        feature_names=feature_names,
        target_label=target_label,
        graph_feature_groups=GRAPH_FEATURE_GROUPS.copy(),
    )


def feature_schema_hash(schema: ArtifactFeatureSchema, feature_set_version: str) -> str:
    validate_feature_names(schema.feature_names)
    document = {
        "feature_set_version": feature_set_version,
        "feature_names": schema.feature_names,
        "target_label": schema.target_label.value,
    }
    if schema.graph_feature_groups:
        document["graph_feature_groups"] = [
            group.value for group in schema.graph_feature_groups
        ]
    digest = hashlib.sha256(repr(document).encode("utf-8")).hexdigest()
    return f"sha256:{digest}"
