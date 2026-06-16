from __future__ import annotations

from dataclasses import dataclass


def future_return_6h(entry_price: float, future_exit_price: float) -> float:
    if entry_price <= 0:
        raise ValueError("entry_price must be positive")
    return (future_exit_price - entry_price) / entry_price


def future_tax_adjusted_return_6h(
    entry_price: float,
    future_exit_price: float,
    tax_rate: float,
) -> float:
    gross_return = future_return_6h(entry_price, future_exit_price)
    gross_profit_pct = max(gross_return, 0.0)
    return gross_return - gross_profit_pct * tax_rate


@dataclass(frozen=True)
class ActionabilityPenaltyInputs:
    estimated_tax_rate: float
    estimated_slippage_pct: float
    liquidity_penalty_pct: float


def future_actionable_return_6h(
    entry_price: float,
    future_exit_price: float,
    penalties: ActionabilityPenaltyInputs,
) -> float:
    adjusted = future_tax_adjusted_return_6h(
        entry_price,
        future_exit_price,
        penalties.estimated_tax_rate,
    )
    return adjusted - penalties.estimated_slippage_pct - penalties.liquidity_penalty_pct
