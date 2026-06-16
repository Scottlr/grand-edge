from __future__ import annotations

from dataclasses import dataclass

import polars as pl


@dataclass(frozen=True)
class MetaLabelTrainingSummary:
    strategy_id: str
    row_count: int
    label_name: str
    model_backend: str


def smoke_train_meta_label(
    features: pl.DataFrame,
    *,
    strategy_id: str,
    label_name: str,
) -> MetaLabelTrainingSummary:
    return MetaLabelTrainingSummary(
        strategy_id=strategy_id,
        row_count=features.height,
        label_name=label_name,
        model_backend="fixture-smoke-only",
    )
