# No-Docker Local Setup

## Prerequisites

- Rust with `rustup`
- Node.js 20+
- npm
- A reachable Postgres 16+ instance
- Optional: `uv` for the ML workflow

## Environment

Copy the shared templates only when you need local overrides:

```powershell
Copy-Item .env.example .env
Copy-Item configs/local.example.toml configs/local.toml
```

Default host-local values:

- `DATABASE_URL=postgres://grand_edge:grand_edge@localhost:5432/grand_edge`
- `VITE_API_BASE_URL=http://localhost:3000`
- `GRAND_EDGE_PROFILE=local`

## Start Postgres locally

Use your normal local Postgres install or any reachable host. Docker is not
required for this path.

## Migrate database

```powershell
cargo run -p grand-edge-xtask -- db migrate
```

## Run backend

```powershell
cargo run -p grand-edge-xtask -- server run --profile local
```

## Run frontend

```powershell
npm --prefix apps/web install
npm --prefix apps/web run dev
```

## Run one ingestion smoke

```powershell
cargo run -p grand-edge-xtask -- ingest latest --profile local
```

## Open the app

- API health: `http://localhost:3000/health`
- OpenAPI: `http://localhost:3000/api/openapi.json`
- Frontend: `http://localhost:5173`

## Stop processes

Stop the backend and frontend terminals directly with `Ctrl+C`.

## Common failures

- `grandedge.exe` blocked by Windows App Control:
  Run `pwsh ./scripts/dev/grandedge-dev.ps1 doctor` and address the policy.
- `uv` missing:
  Install it or skip ML-specific commands for now.
- Port conflict on `3000` or `5173`:
  Stop the conflicting process or update local config and frontend env values.
- Postgres connection refused:
  Recheck `DATABASE_URL` and whether your local server is listening.
