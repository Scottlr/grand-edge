from __future__ import annotations

from dataclasses import dataclass

import polars as pl


@dataclass(frozen=True)
class TrainingSummary:
    strategy_id: str
    row_count: int
    feature_count: int
    target_label: str
    model_backend: str


def smoke_train_gbm(
    features: pl.DataFrame,
    *,
    strategy_id: str,
    target_label: str,
) -> TrainingSummary:
    return TrainingSummary(
        strategy_id=strategy_id,
        row_count=features.height,
        feature_count=len(features.columns),
        target_label=target_label,
        model_backend="fixture-smoke-only",
    )
