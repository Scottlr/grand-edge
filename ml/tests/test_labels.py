from grandedge_ml.labels import (
    ActionabilityPenaltyInputs,
    future_actionable_return_6h,
    future_tax_adjusted_return_6h,
)


def test_future_tax_adjusted_return_6h() -> None:
    value = future_tax_adjusted_return_6h(
        entry_price=100.0,
        future_exit_price=120.0,
        tax_rate=0.02,
    )

    assert round(value, 4) == 0.196


def test_future_actionable_return_subtracts_tax_slippage_and_liquidity_penalty() -> None:
    penalties = ActionabilityPenaltyInputs(
        estimated_tax_rate=0.02,
        estimated_slippage_pct=0.01,
        liquidity_penalty_pct=0.03,
    )

    value = future_actionable_return_6h(
        entry_price=100.0,
        future_exit_price=120.0,
        penalties=penalties,
    )

    assert round(value, 4) == 0.156
