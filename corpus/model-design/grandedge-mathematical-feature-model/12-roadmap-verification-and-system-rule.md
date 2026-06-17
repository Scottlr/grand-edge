# 34. Minimum Viable Model Order

Build in this order:

## Phase 1: Deterministic Features

```text
mid
spread
returns
rolling mean/std
z-score
volume
volume z-score
volatility
tax-adjusted edge
fill probability
data quality
```

## Phase 2: Rule-Based Strategies

```text
spread_edge_v1
momentum_v1
mean_reversion_v1
liquidity_filter_v1
volatility_filter_v1
```

## Phase 3: Statistical Models

```text
kalman_fair_value_v1
ar1_baseline_v1
logistic_direction_v1
ewma_volatility_v1
regime_rules_v1
```

## Phase 4: ML Models

```text
gbm_ranker_v1
meta_label_v1
calibration_v1
conformal_interval_v1
contextual_bandit_v1
online_ensemble_v1
```

## Phase 5: Portfolio and User Models

```text
cashout_model_v1
position_risk_model_v1
portfolio_allocator_v1
user_risk_preference_v1
```

---

# 35. Hard Verification Rules

Every feature/model must have fixture tests.

## Required Tests

```text
mid price test
spread test
spread percentage test
simple return test
log return test
rolling mean/std test
z-score test
EMA test
EWMA volatility test
GARCH step test
tax test
net profit test
ROI test
fill capacity test
fill probability test
Kalman update test
AR(1) forecast test
Brier score test
Hedge update test
Triple barrier test
Recommendation confidence test
Recommendation score test
```

## Example Test Fixture

```json
{
  "name": "tax_adjusted_profit",
  "input": {
    "buyPrice": 100000,
    "sellPrice": 103000,
    "taxRate": 0.02,
    "taxCap": 5000000,
    "slippage": 0
  },
  "expected": {
    "tax": 2060,
    "netProfit": 940,
    "roi": 0.0094
  }
}
```

---

# 36. Final System Rule

For every recommendation, the system must be able to reconstruct:

```text
what data was used
what features were calculated
which models ran
what each model predicted
how predictions were combined
why the recommendation was made
how confident the system was
what invalidates the recommendation
what actually happened later
whether the reasoning was good
```

That is the mathematical and architectural basis of a serious recommendation engine.
