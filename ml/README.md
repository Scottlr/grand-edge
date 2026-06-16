# Grand Edge ML

This workspace is research-only.

Python may load Rust-produced feature exports, train offline models, evaluate
them, and export artifact bundles for Rust validation. Python must not serve
live recommendations, paper trades, API routes, or dashboard streams.

Use `uv` for environment management when available:

- `uv sync --project ml`
- `uv run --project ml ruff check .`
- `uv run --project ml pytest`
- `uv run --project ml python -m grandedge_ml.export --help`

Optional training dependencies are behind `--extra training`.
