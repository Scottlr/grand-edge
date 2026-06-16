# Schema Export

Grand Edge treats contract schemas as reviewed artifacts, not throwaway build
outputs. Rust owns the source types; schema export collects those types into
stable JSON files for Python, docs, CI, and future agent workflows.

## Command

```powershell
cargo run -p grand-edge-xtask -- schema export --out schemas
```

The command writes committed files under `schemas/`:

- `artifact_feature_schema_document.schema.json`
- `model_artifact_metadata.schema.json`
- `model_card_document.schema.json`
- `recommendation_explanation.schema.json`
- `risk_config.schema.json`
- `strategy_config.schema.json`
- `structured_recommendation_explanation.schema.json`
- `openapi.json`
- `schema-manifest.json`

## Review Policy

Generated schemas are committed. If a Rust contract changes intentionally:

1. update the owning Rust type;
2. run the export command again;
3. review the JSON diff like any other public contract change;
4. call out compatibility impact in the PR when consumers may break.

`schema-manifest.json` records the exported schema names, relative file paths,
generator version, timestamp, and SHA-256 hashes. Paths must stay relative and
must not include machine-local directories.

Snapshot tests in `crates/xtask` fail when schema output shifts unexpectedly.
That gives us a review gate before Python or other consumers drift silently.

## Consumer Split

Use JSON Schema for typed payload validation outside the live API:

- strategy config and risk config review;
- model artifact metadata and document validation;
- recommendation explanation shape checks in downstream tooling.

Use OpenAPI for HTTP surfaces:

- route documentation;
- client generation;
- local browser inspection through Swagger UI.

JSON Schema does not replace semantic validation. Artifact bundles must still
pass Rust runtime checks for time windows, feature hashes, target labels, and
file integrity.

## Swagger UI Policy

Swagger UI is mounted at `/swagger-ui/` only when
`api.swagger_ui_enabled = true`.

- Local/dev config: keep it enabled for contract inspection.
- Production-facing config: disable it unless you explicitly want public API
  browsing.

The OpenAPI document itself remains available at `/api/openapi.json`.
