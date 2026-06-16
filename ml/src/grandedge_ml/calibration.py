from __future__ import annotations

from datetime import UTC, datetime

from .artifact_schema import CalibrationMetadata


def build_calibration_metadata(
    *,
    method: str = "identity",
    bins: list[dict[str, float]] | None = None,
    fitted_at: datetime | None = None,
) -> CalibrationMetadata:
    return CalibrationMetadata(
        method=method,
        fitted_at=fitted_at or datetime.now(UTC),
        bins=bins or [],
    )
