from __future__ import annotations

import hashlib
from dataclasses import asdict, dataclass

from .artifact_schema import ArtifactFeatureSchema, TrainingTargetLabel

FEATURE_SET_VERSION = "features_v1"
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


@dataclass(frozen=True)
class FeatureSchemaDocument:
    feature_set_version: str
    feature_names: list[str]
    target_label: str


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
    )


def feature_schema_hash(schema: ArtifactFeatureSchema, feature_set_version: str) -> str:
    validate_feature_names(schema.feature_names)
    document = FeatureSchemaDocument(
        feature_set_version=feature_set_version,
        feature_names=schema.feature_names,
        target_label=schema.target_label.value,
    )
    digest = hashlib.sha256(repr(asdict(document)).encode("utf-8")).hexdigest()
    return f"sha256:{digest}"
