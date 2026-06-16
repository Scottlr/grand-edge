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
2. Copy `.env.example` to `.env` and adjust `DATABASE_URL` if your Postgres host
   differs.
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
- `cargo run -p grand-edge-api`

The API binary is a placeholder today and will be expanded in later tasks.

## Frontend Commands

- `npm --prefix apps/web install`
- `npm --prefix apps/web run dev`
- `npm --prefix apps/web run typecheck`
- `npm --prefix apps/web run build`
- `npm --prefix apps/web run preview`
- `npm --prefix apps/web run lint`

## Environment

The required scaffold-time environment keys live in `.env.example`:

- `DATABASE_URL`
- `GRAND_EDGE_USER_AGENT`
- `OSRS_WIKI_BASE_URL`
- `INGEST_POLL_SECONDS`
- `VITE_API_BASE_URL`

`GRAND_EDGE_USER_AGENT` must remain descriptive and include the project name and
contact address for future OSRS Wiki API access.

## Planning

The current source of truth for implementation order and architectural
invariants is
`features/rust-backed-osrs-recommendation-terminal/tasks.md` together with the
task detail files under
`features/rust-backed-osrs-recommendation-terminal/tasks/`.
