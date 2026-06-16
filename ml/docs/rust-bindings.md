# Rust Research Bindings

`grandedge_py` is a research-only PyO3 module. Python imports Rust helpers for
offline notebooks and experiments; production Rust does not import Python.

## Workflow

- `cargo check -p grandedge_py`
- `cargo check --workspace --exclude grandedge_py`
- `cd ml && uv run --project . maturin develop`
- `cd ml && uv run --project . pytest tests/test_rust_bindings.py`

If the uv-managed venv does not include `pip`, seed it once before the develop
step:

- `cd ml && uv run --project . python -m ensurepip`

## Exposed Functions

### `tax_for_sale(market_rules_json, item_id, sell_price_gp) -> int`

`market_rules_json` is the JSON form of `MarketRules`.

Example:

```python
rules_json = """{
  "version": "osrs_rules_v1_review_required",
  "tax_rate": 0.02,
  "tax_cap_gp": 5000000,
  "tax_min_price_gp": 100,
  "slot_limit": 8,
  "buy_limit_window_secs": 14400,
  "tax_exempt_item_ids": []
}"""

tax_gp = grandedge_py.tax_for_sale(rules_json, 4151, 103_000)
```

### `spread_features_from_json(feature_input_json) -> str`

`feature_input_json` must encode:

```json
{
  "input": {
    "item": { "...": "grand_edge_domain::Item JSON" },
    "latest": { "...": "grand_edge_domain::LatestPrice JSON" },
    "interval_5m": [{ "...": "grand_edge_domain::IntervalPrice JSON" }],
    "interval_1h": [{ "...": "grand_edge_domain::IntervalPrice JSON" }],
    "as_of": "2026-06-16T12:00:00Z",
    "graph_context": null
  },
  "config": {
    "rolling_window_5m": 12,
    "rolling_window_1h": 24,
    "ewma_lambda": 0.94,
    "stale_after_secs": 900,
    "graph_version": null,
    "graph": {
      "max_graph_depth": 2,
      "min_edge_confidence": 0.55,
      "upstream_lambda": 0.35,
      "downstream_lambda": 0.25,
      "sector_lambda": 0.15,
      "stale_after_secs": 900
    }
  }
}
```

If `config` is omitted, Rust uses `FeatureEngineConfig::default()`.

### `simulate_order_from_json(request_json, history_json) -> str`

`request_json` must encode:

```json
{
  "request": {
    "run_id": "00000000-0000-0000-0000-000000000000",
    "strategy_id": "spread_edge_v1",
    "model_version": "v1",
    "item_id": 4151,
    "created_at": "2026-06-16T12:00:00Z",
    "side": "buy",
    "quantity": 20,
    "limit_price": 100000,
    "target_exit": 103000,
    "stop_loss": 99000,
    "horizon_secs": 21600
  },
  "config": {
    "execution_mode": "passive_estimated",
    "market_rules": { "...": "MarketRules JSON" },
    "participation_rate": 0.05,
    "confidence_haircut": 0.5,
    "default_horizon_secs": 21600,
    "emergency_exit_slippage_gp": 0,
    "worst_case_slippage_gp": 0
  }
}
```

`history_json` is a JSON array of `IntervalPrice` rows. If `config` is omitted,
Rust uses `SimulatorConfig::default()`.

## Error Handling

- Invalid JSON input raises `ValueError`.
- Rust feature/simulation validation errors raise `RuntimeError`.
- The bindings do not open network or database connections and do not read live
  environment configuration.
