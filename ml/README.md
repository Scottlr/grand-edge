# Grand Edge ML

This workspace is research-only.

Python may load Rust-produced feature exports, train offline models, evaluate
them, and export artifact bundles for Rust validation. Python must not serve
live recommendations, paper trades, API routes, or dashboard streams.

The normal research data source is Rust-owned analytics/report exports, not
direct production database queries. Generate datasets and backtest artifacts
through `grandedge` first, then point Python tooling at the exported Parquet
and JSON manifest files:

- `cargo run -p grand-edge-xtask -- analytics export-features --from 2026-01-01 --to 2026-02-01 --out reports/datasets/jan --include-raw-interval-candles`
- `cargo run -p grand-edge-xtask -- backtest report --run-id <uuid> --out reports/backtests/<id>`

Use `uv` for environment management when available:

- `uv sync --project ml`
- `uv run --project ml ruff check .`
- `uv run --project ml pytest`
- `uv run --project ml python -m grandedge_ml.export --help`

Optional training dependencies are behind `--extra training`.
