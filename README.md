# Grand Edge

Grand Edge is a Rust-backed OSRS Grand Exchange recommendation and paper-trading
terminal. This repository currently contains the buildable foundation for the
workspace, local development commands, and the initial frontend shell.

## Run Locally

No Docker:

- `pwsh ./scripts/dev/grandedge-dev.ps1 doctor`
- `pwsh ./scripts/dev/grandedge-dev.ps1 no-docker`

Docker:

- `pwsh ./scripts/dev/grandedge-dev.ps1 docker-up`

ML artifact loop:

- `pwsh ./scripts/dev/grandedge-dev.ps1 ml-export-fixture`
- `pwsh ./scripts/dev/grandedge-dev.ps1 ml-validate-artifact`

Runbooks:

- [No-Docker local setup](/C:/Users/scott/OneDrive/Documents/grand-edge/docs/running/no-docker.md:1)
- [Docker setup](/C:/Users/scott/OneDrive/Documents/grand-edge/docs/running/docker.md:1)
- [ML workflow](/C:/Users/scott/OneDrive/Documents/grand-edge/docs/running/ml-workflow.md:1)
- [Troubleshooting](/C:/Users/scott/OneDrive/Documents/grand-edge/docs/running/troubleshooting.md:1)

Expected URLs:

- API health: `http://localhost:3000/health`
- OpenAPI: `http://localhost:3000/api/openapi.json`
- Frontend: `http://localhost:5173`

## Architecture

- `crates/domain`: shared cross-crate contracts and future newtypes.
- `crates/ingest`: the future OSRS Wiki ingestion boundary.
- `crates/storage`: migrations and persistence ownership.
- `crates/features`: deterministic feature generation.
- `crates/strategies`: strategy traits and signal modules.
- `crates/recommender`: recommendation selection and explanation logic.
- `crates/simulator`: paper-trading replay and fill rules.
- `crates/metrics`: forecasting, risk, and trading evaluation.
- `crates/api`: Axum API and live dashboard event surface.
- `apps/web`: React + TypeScript + Vite terminal UI.

## Run Modes

Two local development paths are supported:

1. No-Docker local setup: Rust, Node.js, and a reachable Postgres instance run
   on the host or on infrastructure you can access directly.
2. Docker-assisted local setup: Docker Compose can run Postgres, the Rust API,
   and the Vite frontend together for local verification.

See the detailed runbooks under `docs/running/` for exact steps.

## Backend Commands

- `cargo fmt --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo test --workspace -- --ignored`
- `cargo bench --workspace --no-run`
- `cargo run -p grand-edge-api`
- `cargo run -p grand-edge-xtask -- --help`
- `cargo run -p grand-edge-xtask -- doctor`
- `cargo run -p grand-edge-xtask -- db migrate`
- `cargo run -p grand-edge-xtask -- server run --profile local`
- `cargo run -p grand-edge-xtask -- ingest latest --profile local`
- `cargo run -p grand-edge-xtask -- config print --profile local`
- `cargo run -p grand-edge-xtask -- schema export --out schemas`
- `cargo run -p grand-edge-xtask -- analytics export-features --from 2026-01-01 --to 2026-02-01 --out reports/datasets/jan --include-raw-interval-candles`
- `cargo run -p grand-edge-xtask -- backtest report --run-id <uuid> --out reports/backtests/<id>`
- `cargo run -p grand-edge-xtask -- model validate --artifact ml/artifacts/fixture`
- `cargo run -p grand-edge-xtask -- model evaluate --strategy gbm_ranker_v1 --version 2026-06-16.1 --artifact ml/artifacts/gbm_ranker_v1/2026-06-16.1`

The API binary is a placeholder today and will be expanded in later tasks.

## Test Harnesses

- Deterministic fixtures live under `tests/fixtures/` and crate-local `tests/fixtures/`.
- Network-free OSRS Wiki client coverage should use `wiremock`, not live API calls.
- Docker-backed storage reality checks live in ignored `testcontainers` tests.
- Snapshot coverage uses `insta` for explanation and API-shape assertions.
- Bench targets compile with `cargo bench --workspace --no-run` and serve as
  stable baselines rather than immediate optimization proofs.

## Frontend Commands

- `npm --prefix apps/web install`
- `npm --prefix apps/web run dev`
- `npm --prefix apps/web run typecheck`
- `npm --prefix apps/web run build`
- `npm --prefix apps/web run preview`
- `npm --prefix apps/web run lint`

## Environment

The shared runtime configuration now layers `configs/default.toml`, optional
`configs/{profile}.toml`, optional `configs/local.toml`, and environment
overrides from `.env.example`:

- `DATABASE_URL`
- `GRAND_EDGE_PROFILE`
- `GRAND_EDGE_USER_AGENT`
- `OSRS_WIKI_BASE_URL`
- `GRAND_EDGE__API__BIND_ADDR`
- `VITE_API_BASE_URL`

`GRAND_EDGE_USER_AGENT` must remain descriptive and include the project name and
contact address for future OSRS Wiki API access.

## Contract Schemas

Rust-owned contract exports live under `schemas/` and are generated with:

- `cargo run -p grand-edge-xtask -- schema export --out schemas`

This writes JSON Schemas for strategy config, risk config, artifact metadata,
artifact documents, and recommendation explanation contracts, plus
`schemas/openapi.json` and `schemas/schema-manifest.json`.

The local API also serves Swagger UI at `/swagger-ui/` when
`api.swagger_ui_enabled = true`. Leave that enabled for local/dev review, and
disable it in production-facing config where public API browsing is not wanted.

## Planning

The current source of truth for implementation order and architectural
invariants is
`features/rust-backed-osrs-recommendation-terminal/tasks.md` together with the
task detail files under
`features/rust-backed-osrs-recommendation-terminal/tasks/`.

## ML Research Workspace

The repository now includes a research-only Python workspace under `ml/`.
Python may load Rust-produced dataset exports, train offline experiments, and
export artifact bundles for Rust validation, but it must not serve live
recommendations or become a runtime dependency of the Rust production path.

Use `uv` for the ML workflow:

- `uv sync --project ml`
- `uv run --project ml ruff check .`
- `uv run --project ml pytest`
- `uv run --project ml python -m grandedge_ml.export --help`

The intended handoff is Rust analytics export -> Python artifact export -> Rust
artifact validation/runtime. See [docs/running/ml-workflow.md](/C:/Users/scott/OneDrive/Documents/grand-edge/docs/running/ml-workflow.md:1).
