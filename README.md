# Grand Edge

Grand Edge is a Rust-backed OSRS Grand Exchange recommendation and paper-trading
terminal. This repository currently contains the buildable foundation for the
workspace, local development commands, and the initial frontend shell.

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

Two local development paths are supported in this scaffold:

1. No-Docker local setup: Rust, Node.js, and a reachable Postgres instance run
   on the host or on infrastructure you can access directly.
2. Docker-assisted local setup: Docker Compose starts only Postgres for local
   development convenience. Application containers are intentionally deferred to
   T045.

Docker is optional in this task. It is not the only supported workflow.

## No-Docker Local Setup

1. Install Rust with `rustup`, Node.js 20+, npm, and ensure a Postgres 16+
   instance is reachable.
2. Copy `configs/local.example.toml` to `configs/local.toml` for file-based local
   overrides, then copy `.env.example` to `.env` only if you need environment
   overrides such as `DATABASE_URL`.
3. Run `cargo check --workspace` from the repository root.
4. Run `npm --prefix apps/web install`.
5. Run `npm --prefix apps/web run dev` to start the frontend shell.

## Docker-Assisted Local Setup

1. Install Docker Desktop or another Docker engine with Compose support.
2. Start Postgres with `docker compose -f docker-compose.dev.yml up -d`.
3. Copy `.env.example` to `.env` and keep `DATABASE_URL` pointed at
   `localhost:5432` unless you changed the Compose ports.
4. Run backend and frontend commands on the host using the sections below.

## Backend Commands

- `cargo fmt --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo test --workspace -- --ignored`
- `cargo bench --workspace --no-run`
- `cargo run -p grand-edge-api`
- `cargo run -p grand-edge-xtask -- --help`
- `cargo run -p grand-edge-xtask -- config print --profile local`
- `cargo run -p grand-edge-xtask -- schema export --out schemas`
- `cargo run -p grand-edge-xtask -- analytics export-features --from 2026-01-01 --to 2026-02-01 --out reports/datasets/jan --include-raw-interval-candles`
- `cargo run -p grand-edge-xtask -- backtest report --run-id <uuid> --out reports/backtests/<id>`

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
- `GRAND_EDGE_USER_AGENT`
- `OSRS_WIKI_BASE_URL`
- `INGEST_POLL_SECONDS`
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
