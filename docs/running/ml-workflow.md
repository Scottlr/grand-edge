# ML Workflow

## Boundary: Rust production, Python research

Grand Edge keeps Python on the research side of the boundary. Python may export
artifacts and run offline experiments, but the live API, recommender, simulator,
strategy registry, ingestion jobs, and dashboard stream stay Rust-only.

## Export a Rust dataset

```powershell
cargo run -p grand-edge-xtask -- analytics export-features --from 2026-01-01 --to 2026-02-01 --out reports/datasets/jan --include-raw-interval-candles
```

If Windows App Control blocks `grandedge.exe` on your machine, run the doctor
script first and fix the policy before relying on local runtime commands:

```powershell
pwsh ./scripts/dev/grandedge-dev.ps1 doctor
```

## Train or export a fixture artifact in Python

```powershell
uv sync --project ml
uv run --project ml python -m grandedge_ml.export --fixture --out ml/artifacts/fixture
uv run --project ml python -m grandedge_ml.export --dataset reports/datasets/jan --out ml/artifacts/gbm_ranker_v1/2026-06-16.1
```

## Validate the artifact in Rust

```powershell
cargo run -p grand-edge-xtask -- model validate --artifact ml/artifacts/gbm_ranker_v1/2026-06-16.1
```

## Evaluate/backtest the artifact in Rust

```powershell
cargo run -p grand-edge-xtask -- model evaluate --strategy gbm_ranker_v1 --version 2026-06-16.1 --artifact ml/artifacts/gbm_ranker_v1/2026-06-16.1
```

## Load artifact-backed strategies

Artifact-backed strategies stay disabled by default until the exported contract
validates and the runtime accepts the artifact. The frontend Strategy Lab makes
that lock state visible instead of implying the model is live.

## What not to do

- Do not start Python from the API server.
- Do not make the recommender or simulator import Python code.
- Do not treat research notebooks as production runtime entrypoints.
- Do not bypass Rust artifact validation just because Python export succeeded.
