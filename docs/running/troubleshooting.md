# Troubleshooting

## Port conflicts

- API default: `3000`
- Frontend default: `5173`
- Postgres default: `5432`

If any are in use, free the port or update your local config and env values
together.

## Database connection errors

- Check `DATABASE_URL`
- Confirm Postgres is listening
- Run `cargo run -p grand-edge-xtask -- db migrate`

## OSRS Wiki user-agent validation failures

Grand Edge requires a descriptive user agent. Keep
`GRAND_EDGE_USER_AGENT=GrandEdge/0.1 (OSRS Grand Exchange recommendation terminal; contact: scott.rangeley@outlook.com)`
or an equivalent descriptive override.

## Stale Vite env values

If the frontend keeps calling the wrong API URL, stop the dev server and restart
it after updating `.env`.

## Missing `uv`

Install `uv` before using ML workflow commands, or skip the research path until
it is available.

## Docker volume resets

Use:

```powershell
docker compose -f docker-compose.dev.yml down -v
```

## Artifact schema mismatch

Run:

```powershell
cargo run -p grand-edge-xtask -- schema export --out schemas
cargo run -p grand-edge-xtask -- model validate --artifact ml/artifacts/fixture
```
