# ML Workflow

Grand Edge keeps Python on the research side of the boundary.

The intended handoff is:

1. Rust ingestion, storage, and feature jobs produce normalized market data and
   `features_v1` exports.
2. Python loads those exported datasets, computes documented research labels such
   as `future_tax_adjusted_return_6h` and `future_actionable_return_6h`, and
   trains offline experiments.
3. Python exports an artifact bundle with `model.onnx`, `model_card.json`,
   `feature_schema.json`, and `calibration.json`.
4. Rust validates artifact metadata and feature-schema compatibility before any
   runtime model serving work. The current Rust-side source of truth is
   [crates/strategies/src/artifacts.rs](/C:/Users/scott/OneDrive/Documents/grand-edge/crates/strategies/src/artifacts.rs:1).

Use the research workspace with `uv`:

```powershell
uv sync --project ml
uv run --project ml ruff check .
uv run --project ml pytest
uv run --project ml python -m grandedge_ml.export --help
uv run --project ml python -m grandedge_ml.export --output-root ml/artifacts
```

Optional training libraries remain opt-in:

```powershell
uv sync --project ml --extra training
```

This document is the interim workflow for T021. The broader local/Docker/ML
runbook remains owned by T045.
