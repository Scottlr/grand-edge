from __future__ import annotations

import json
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path


@dataclass(frozen=True)
class EvaluationWindow:
    training_window_start: datetime
    training_window_end: datetime
    evaluation_window_start: datetime
    evaluation_window_end: datetime
    as_of: datetime


def validate_evaluation_window(window: EvaluationWindow) -> None:
    training_start = window.training_window_start.astimezone(UTC)
    training_end = window.training_window_end.astimezone(UTC)
    evaluation_start = window.evaluation_window_start.astimezone(UTC)
    evaluation_end = window.evaluation_window_end.astimezone(UTC)
    as_of = window.as_of.astimezone(UTC)

    if training_end < training_start:
        raise ValueError("training window end must be after start")
    if evaluation_start < training_end:
        raise ValueError("evaluation window must start after training window end")
    if evaluation_end < evaluation_start:
        raise ValueError("evaluation window end must be after evaluation start")
    if training_end > as_of or evaluation_end > as_of:
        raise ValueError("training/evaluation windows must not extend past as_of")


def summarize_directional_accuracy(actual: list[float], predicted: list[float]) -> float | None:
    if len(actual) != len(predicted) or not actual:
        return None
    matches = sum(
        1
        for observed, forecast in zip(actual, predicted, strict=True)
        if (observed >= 0) == (forecast >= 0)
    )
    return matches / len(actual)


def write_report(path: str | Path, metrics: dict[str, float | int | str | None]) -> Path:
    report_path = Path(path)
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(metrics, indent=2) + "\n", encoding="utf-8")
    return report_path
