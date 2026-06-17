# 27. Portfolio Allocation

## 27.1 Purpose

Chooses how much capital to allocate across candidate items.

## 27.2 Expected Value

```text
EV = p_win * avg_win - p_loss * avg_loss
```

### Example

Input:

```json
{
  "pWin": 0.60,
  "avgWin": 20000,
  "pLoss": 0.40,
  "avgLoss": 10000
}
```

Output:

```json
{
  "expectedValue": 8000
}
```

## 27.3 Kelly Fraction

Aggressive sizing formula:

```text
f* = (bp - q) / b
```

Where:

```text
b = win_profit / loss_amount
p = probability_win
q = 1 - p
```

Use fractional Kelly only, such as 10% or 25%, because pure Kelly is too aggressive.

### Example

Input:

```json
{
  "winProfit": 20000,
  "lossAmount": 10000,
  "p": 0.60
}
```

Calculation:

```text
b = 2
q = 0.4
f = (2*0.6 - 0.4) / 2 = 0.4
fractional_25_percent_kelly = 0.1
```

Output:

```json
{
  "kellyFraction": 0.4,
  "recommendedFraction": 0.1
}
```

## 27.4 Rust

```rust
pub fn expected_value(p_win: f64, avg_win: f64, avg_loss: f64) -> f64 {
    let p_loss = 1.0 - p_win;

    p_win * avg_win - p_loss * avg_loss
}

pub fn kelly_fraction(p_win: f64, win_profit: f64, loss_amount: f64) -> Option<f64> {
    if loss_amount <= 0.0 {
        return None;
    }

    let b = win_profit / loss_amount;
    let q = 1.0 - p_win;

    if b <= 0.0 {
        return None;
    }

    Some(((b * p_win - q) / b).clamp(0.0, 1.0))
}
```

## 27.5 Constrained Portfolio Optimisation

Objective:

```text
maximise Σ quantity_i * expected_net_profit_i - λ * portfolio_risk
```

Subject to:

```text
Σ quantity_i * entry_price_i <= capital
quantity_i <= buy_limit_i
quantity_i <= estimated_fill_capacity_i
active_items <= slot_limit
```

For MVP, a greedy allocator is acceptable:

```text
rank by expected_net_profit_per_gp
allocate until capital or slot limit exhausted
```

---

# 28. Simulation Metrics

## 28.1 Realised Return

```text
realised_return = (exit_price - tax - entry_price) / entry_price
```

## 28.2 Max Favourable Excursion

Best unrealised move during holding period:

```text
MFE = max(price_path - entry_price) / entry_price
```

## 28.3 Max Adverse Excursion

Worst unrealised move during holding period:

```text
MAE = min(price_path - entry_price) / entry_price
```

## 28.4 Drawdown

```text
drawdown_t = (equity_t - peak_equity_t) / peak_equity_t
```

Maximum drawdown:

```text
max_drawdown = min(drawdown_t)
```

## 28.5 Profit Factor

```text
profit_factor = gross_wins / abs(gross_losses)
```

## 28.6 Directional Accuracy

```text
directional_accuracy = correct_direction_predictions / total_predictions
```

## 28.7 Rust: Directional Accuracy

```rust
pub fn directional_accuracy(predicted_up: &[bool], actual_up: &[bool]) -> Option<f64> {
    if predicted_up.len() != actual_up.len() || predicted_up.is_empty() {
        return None;
    }

    let correct = predicted_up
        .iter()
        .zip(actual_up.iter())
        .filter(|(predicted, actual)| predicted == actual)
        .count();

    Some(correct as f64 / predicted_up.len() as f64)
}
```

---
