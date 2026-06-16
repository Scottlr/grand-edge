from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

import polars as pl


@dataclass(frozen=True)
class DatasetSlice:
    path: Path
    row_count: int
    columns: tuple[str, ...]


def load_feature_frame(path: str | Path) -> pl.DataFrame:
    dataset_path = Path(path)
    if not dataset_path.exists():
        raise FileNotFoundError(dataset_path)
    if dataset_path.suffix == ".parquet":
        return pl.read_parquet(dataset_path)
    if dataset_path.suffix == ".csv":
        return pl.read_csv(dataset_path)
    raise ValueError(f"unsupported dataset suffix: {dataset_path.suffix}")


def describe_dataset(path: str | Path) -> DatasetSlice:
    frame = load_feature_frame(path)
    return DatasetSlice(
        path=Path(path),
        row_count=frame.height,
        columns=tuple(frame.columns),
    )
