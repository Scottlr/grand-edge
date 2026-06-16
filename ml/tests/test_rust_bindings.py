from __future__ import annotations

import json
import os
from datetime import UTC, datetime, timedelta
from uuid import uuid4

import pytest

grandedge_py = pytest.importorskip(
    "grandedge_py",
    reason="install the research bindings with `uv run --project ml maturin develop`",
)


def iso_z(value: datetime) -> str:
    return value.astimezone(UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def market_rules_json() -> str:
    return json.dumps(
        {
            "version": "osrs_rules_v1_review_required",
            "tax_rate": 0.02,
            "tax_cap_gp": 5_000_000,
            "tax_min_price_gp": 100,
            "slot_limit": 8,
            "buy_limit_window_secs": 14_400,
            "tax_exempt_item_ids": [],
        }
    )


def feature_fixture_request_json() -> str:
    as_of = datetime(2026, 6, 16, 12, 0, tzinfo=UTC)
    interval_5m = [
        {
            "item_id": 4151,
            "bucket_start": iso_z(as_of - timedelta(hours=11 - index)),
            "interval": "five_minute",
            "avg_high_price": 70 + index,
            "high_price_volume": 10 + index,
            "avg_low_price": 60 + index,
            "low_price_volume": 8 + index,
        }
        for index in range(12)
    ]
    interval_1h = [
        {
            "item_id": 4151,
            "bucket_start": iso_z(as_of - timedelta(hours=23 - index)),
            "interval": "one_hour",
            "avg_high_price": 90 + index,
            "high_price_volume": 200 + index * 10,
            "avg_low_price": 80 + index,
            "low_price_volume": 170 + index * 8,
        }
        for index in range(24)
    ]
    return json.dumps(
        {
            "input": {
                "item": {
                    "item_id": 4151,
                    "name": "Abyssal whip",
                    "examine": "A weapon from the abyss.",
                    "members": True,
                    "buy_limit": 70,
                    "low_alch": 48_000,
                    "high_alch": 72_000,
                    "value": 120_001,
                    "icon": None,
                    "updated_at": iso_z(as_of),
                },
                "latest": {
                    "item_id": 4151,
                    "high": 100,
                    "high_time": iso_z(as_of - timedelta(minutes=1)),
                    "low": 80,
                    "low_time": iso_z(as_of - timedelta(minutes=2)),
                    "observed_at": iso_z(as_of),
                },
                "interval_5m": interval_5m,
                "interval_1h": interval_1h,
                "as_of": iso_z(as_of),
                "graph_context": None,
            }
        }
    )


def simulation_request_json() -> str:
    return json.dumps(
        {
            "config": {
                "execution_mode": "passive_estimated",
                "market_rules": json.loads(market_rules_json()),
                "participation_rate": 0.05,
                "confidence_haircut": 0.5,
                "default_horizon_secs": 21_600,
                "emergency_exit_slippage_gp": 0,
                "worst_case_slippage_gp": 0,
            },
            "request": {
                "run_id": str(uuid4()),
                "strategy_id": "spread_edge_v1",
                "model_version": "v1",
                "item_id": 4151,
                "created_at": "2026-06-16T12:00:00Z",
                "side": "buy",
                "quantity": 20,
                "limit_price": 100_000,
                "target_exit": 103_000,
                "stop_loss": 99_000,
                "horizon_secs": 21_600,
            },
        }
    )


def simulation_history_json() -> str:
    return json.dumps(
        [
            {
                "item_id": 4151,
                "bucket_start": "2026-06-16T12:00:00Z",
                "interval": "one_hour",
                "avg_high_price": 102_000,
                "high_price_volume": 250,
                "avg_low_price": 99_000,
                "low_price_volume": 170,
            },
            {
                "item_id": 4151,
                "bucket_start": "2026-06-16T13:00:00Z",
                "interval": "one_hour",
                "avg_high_price": 103_000,
                "high_price_volume": 250,
                "avg_low_price": 100_001,
                "low_price_volume": 170,
            },
            {
                "item_id": 4151,
                "bucket_start": "2026-06-16T14:00:00Z",
                "interval": "one_hour",
                "avg_high_price": 104_000,
                "high_price_volume": 250,
                "avg_low_price": 99_000,
                "low_price_volume": 170,
            },
            {
                "item_id": 4151,
                "bucket_start": "2026-06-16T15:00:00Z",
                "interval": "one_hour",
                "avg_high_price": 103_500,
                "high_price_volume": 250,
                "avg_low_price": 100_500,
                "low_price_volume": 170,
            },
        ]
    )


def test_tax_for_sale_matches_rust_fixture() -> None:
    assert grandedge_py.tax_for_sale(market_rules_json(), 4151, 103_000) == 2_060


def test_spread_features_match_rust_fixture() -> None:
    payload = grandedge_py.spread_features_from_json(feature_fixture_request_json())
    parsed = json.loads(payload)

    assert parsed["feature_set_version"] == "features_v1"
    assert parsed["values"]["spread_abs"] == 20
    assert parsed["values"]["observed_volume_1h"] == 784
    assert parsed["values"]["buy_limit"] == 70


def test_simulate_order_rejects_future_creation_bucket() -> None:
    payload = grandedge_py.simulate_order_from_json(
        simulation_request_json(),
        simulation_history_json(),
    )
    parsed = json.loads(payload)

    assert parsed["entry_time"] == "2026-06-16T14:00:00Z"
    assert parsed["status"] == "closed"


def test_invalid_json_raises_value_error() -> None:
    with pytest.raises(ValueError):
        grandedge_py.spread_features_from_json("{not json")


def test_bindings_do_not_require_database_url(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("DATABASE_URL", raising=False)
    monkeypatch.delenv("PGHOST", raising=False)
    monkeypatch.delenv("PGPORT", raising=False)

    assert "DATABASE_URL" not in os.environ
    assert grandedge_py.tax_for_sale(market_rules_json(), 4151, 103_000) == 2_060
