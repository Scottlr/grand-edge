# 22. Meta-Labeling and Triple Barrier

## 22.1 Purpose

A base model proposes a trade. A meta-model decides whether to accept it.

This prevents weak trades from being taken just because one model says “buy”.

## 22.2 Triple Barrier Label

For an entry price:

```text
upper_barrier = entry_price * (1 + take_profit_pct)
lower_barrier = entry_price * (1 - stop_loss_pct)
vertical_barrier = max_holding_time
```

Label:

```text
1  if upper barrier hit first
-1 if lower barrier hit first
0  if neither hit before timeout
```

## 22.3 Example

Input:

```json
{
  "entry": 100,
  "takeProfitPct": 0.03,
  "stopLossPct": 0.02,
  "futurePath": [99, 101, 103.2]
}
```

Output:

```json
{
  "upperBarrier": 103,
  "lowerBarrier": 98,
  "label": 1
}
```

## 22.4 Rust

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TripleBarrierLabel {
    TakeProfit,
    StopLoss,
    Timeout,
}

pub fn triple_barrier_label(
    entry: f64,
    take_profit_pct: f64,
    stop_loss_pct: f64,
    future_prices: &[f64],
) -> TripleBarrierLabel {
    let upper = entry * (1.0 + take_profit_pct);
    let lower = entry * (1.0 - stop_loss_pct);

    for price in future_prices {
        if *price >= upper {
            return TripleBarrierLabel::TakeProfit;
        }

        if *price <= lower {
            return TripleBarrierLabel::StopLoss;
        }
    }

    TripleBarrierLabel::Timeout
}
```

## 22.5 Meta-Model Target

```text
y = 1 if base signal produced acceptable realised return
y = 0 otherwise
```

Features:

```text
base model confidence
spread_pct
volatility
volume_z
regime
recent model accuracy
fill_probability
```

Output:

```json
{
  "baseSignal": "buy",
  "metaProbabilitySuccess": 0.63,
  "accepted": true
}
```

---

# 23. Conformal Prediction Intervals

## 23.1 Purpose

Gives uncertainty intervals around predictions.

Instead of saying:

```text
expected return = 3%
```

say:

```text
expected return = 3%, likely range 1.2% to 4.8%
```

## 23.2 Formula

On calibration set:

```text
residual_i = |actual_i - predicted_i|
q = quantile(residuals, confidence_level)
```

Prediction interval:

```text
[prediction - q, prediction + q]
```

## 23.3 Example

Input:

```json
{
  "predictedReturn": 0.03,
  "residualQuantile90": 0.018
}
```

Output:

```json
{
  "lower": 0.012,
  "upper": 0.048,
  "coverage": 0.90
}
```

## 23.4 Rust

```rust
pub fn conformal_interval(prediction: f64, residual_quantile: f64) -> (f64, f64) {
    (prediction - residual_quantile, prediction + residual_quantile)
}
```

## 23.5 Python Quantile

```python
import numpy as np

predicted = np.array([0.01, 0.02, -0.01, 0.03])
actual = np.array([0.015, 0.005, -0.02, 0.025])

residuals = np.abs(actual - predicted)
q90 = np.quantile(residuals, 0.90)

new_prediction = 0.03
interval = (new_prediction - q90, new_prediction + q90)

print({
    "q90": float(q90),
    "interval": [float(interval[0]), float(interval[1])]
})
```

## 23.6 What it provides

Used by:

* Forecast band
* Confidence display
* Risk control
* UI honesty

---
