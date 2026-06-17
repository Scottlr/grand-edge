# Docker Setup

## Prerequisites

- Docker Desktop or another Docker engine with Compose support

## Start full stack

```powershell
pwsh ./scripts/dev/grandedge-dev.ps1 docker-up
```

For config-only validation:

```powershell
docker compose -f docker-compose.dev.yml config
```

## Run only Postgres in Docker with host backend/frontend

```powershell
docker compose -f docker-compose.dev.yml up -d postgres
cargo run -p grand-edge-xtask -- db migrate
npm --prefix apps/web run dev
```

## Inspect logs

```powershell
docker compose -f docker-compose.dev.yml logs -f
```

## Reset local Docker database

```powershell
docker compose -f docker-compose.dev.yml down -v
```

## Stop stack

```powershell
pwsh ./scripts/dev/grandedge-dev.ps1 docker-down
```

## Common failures

- `docker` command missing:
  Install Docker Desktop or use the no-Docker path instead.
- API cannot reach Postgres:
  Check the `postgres` service health and the Compose `DATABASE_URL`.
- Frontend points at the wrong API:
  Recheck `VITE_API_BASE_URL`.
