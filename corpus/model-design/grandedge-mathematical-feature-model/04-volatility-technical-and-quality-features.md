# 9. Volatility Features

## 9.1 Realised Volatility

### Purpose

Measures variability of returns.

### Formula

```text
realised_volatility = sqrt(mean(r_t^2))
```

or:

```text
realised_volatility = std(log_returns)
```

### Example

Input:

```json
{
  "logReturns": [0.01, -0.02, 0.015]
}
```

Output approximately:

```json
{
  "realisedVolatility": 0.0147
}
```

### Rust

```rust
pub fn realised_volatility(returns: &[f64]) -> Option<f64> {
    if returns.is_empty() {
        return None;
    }

    let mean_square = returns.iter().map(|r| r * r).sum::<f64>() / returns.len() as f64;

    Some(mean_square.sqrt())
}
```

---

## 9.2 EWMA Volatility

### Purpose

Forecasts volatility while weighting recent shocks more strongly.

### Formula

```text
σ²_t = λσ²_{t-1} + (1 - λ)r²_{t-1}
```

Common λ is around `0.94` in finance, but this should be tuned for OSRS.

### Example

Input:

```json
{
  "lambda": 0.94,
  "previousVariance": 0.0004,
  "lastReturnSquared": 0.0009
}
```

Output:

```json
{
  "variance": 0.00043,
  "volatility": 0.020736
}
```

### Rust

```rust
pub fn ewma_variance(lambda: f64, previous_variance: f64, last_return: f64) -> f64 {
    lambda * previous_variance + (1.0 - lambda) * last_return.powi(2)
}

pub fn ewma_volatility(lambda: f64, previous_variance: f64, last_return: f64) -> f64 {
    ewma_variance(lambda, previous_variance, last_return).sqrt()
}
```

### What it provides

Used by:

* Risk adjustment
* Stop-loss sizing
* Confidence penalty
* Regime detection

---

## 9.3 GARCH(1,1) Volatility

### Purpose

Models volatility clustering.

Useful when large moves tend to be followed by more large moves.

### Formula

```text
σ²_t = ω + αε²_{t-1} + βσ²_{t-1}
```

Where:

* `ω` is baseline variance
* `α` controls reaction to last shock
* `β` controls persistence

### Example

Input:

```json
{
  "omega": 0.00001,
  "alpha": 0.1,
  "beta": 0.85,
  "lastResidual": 0.03,
  "previousVariance": 0.0004
}
```

Calculation:

```text
variance = 0.00001 + 0.1 * 0.0009 + 0.85 * 0.0004
variance = 0.00044
volatility = 0.020976
```

Output:

```json
{
  "variance": 0.00044,
  "volatility": 0.020976
}
```

### Rust

```rust
pub fn garch_11_variance(
    omega: f64,
    alpha: f64,
    beta: f64,
    last_residual: f64,
    previous_variance: f64,
) -> f64 {
    omega + alpha * last_residual.powi(2) + beta * previous_variance
}
```

---

# 10. Technical Indicator Features

These should be used as features, not blindly as trading rules.

## 10.1 Bollinger Band Position

### Purpose

Identifies whether price is high or low relative to recent distribution.

### Formula

```text
upper = mean_n + k * std_n
lower = mean_n - k * std_n
band_position = (price_t - lower) / (upper - lower)
```

### Example

Input:

```json
{
  "price": 90,
  "mean": 100,
  "std": 5,
  "k": 2
}
```

Output:

```json
{
  "upper": 110,
  "lower": 90,
  "bandPosition": 0.0
}
```

### Rust

```rust
pub fn bollinger_band_position(
    price: f64,
    mean: f64,
    std: f64,
    k: f64,
) -> Option<f64> {
    let upper = mean + k * std;
    let lower = mean - k * std;
    let width = upper - lower;

    if width <= 0.0 {
        return None;
    }

    Some((price - lower) / width)
}
```

---

## 10.2 RSI

### Purpose

Measures recent upward versus downward movement.

Useful as a feature for overextension, not as a standalone “buy below 30” rule.

### Formula

```text
RS = average_gain / average_loss
RSI = 100 - (100 / (1 + RS))
```

### Example

Input:

```json
{
  "averageGain": 2,
  "averageLoss": 1
}
```

Output:

```json
{
  "rsi": 66.6667
}
```

### Rust

```rust
pub fn rsi(average_gain: f64, average_loss: f64) -> Option<f64> {
    if average_loss == 0.0 {
        return Some(100.0);
    }

    if average_gain == 0.0 {
        return Some(0.0);
    }

    let rs = average_gain / average_loss;

    Some(100.0 - (100.0 / (1.0 + rs)))
}
```

---

# 11. Data Quality Features

## 11.1 Price Staleness

### Purpose

Prevents confident recommendations on old data.

### Formula

```text
staleness_seconds = now - max(high_time, low_time)
```

### Example

Input:

```json
{
  "now": 1781455600,
  "highTime": 1781455538,
  "lowTime": 1781455543
}
```

Output:

```json
{
  "stalenessSeconds": 57
}
```

### Rust

```rust
pub fn staleness_seconds(now: i64, high_time: Option<i64>, low_time: Option<i64>) -> Option<i64> {
    let latest = match (high_time, low_time) {
        (Some(h), Some(l)) => h.max(l),
        (Some(h), None) => h,
        (None, Some(l)) => l,
        (None, None) => return None,
    };

    Some((now - latest).max(0))
}
```

---

## 11.2 Data Quality Confidence

### Purpose

Determines whether the recommender should trust the data.

### Example formula

```text
data_quality =
  1.0
  - missing_penalty
  - staleness_penalty
  - spread_extreme_penalty
```

Where:

```text
staleness_penalty = min(staleness_seconds / max_allowed_staleness, 1.0) * weight
```

### Rust

```rust
pub fn data_quality_confidence(
    missing_high: bool,
    missing_low: bool,
    staleness_seconds: f64,
    max_staleness_seconds: f64,
    spread_pct: f64,
) -> f64 {
    let mut score = 1.0;

    if missing_high {
        score -= 0.2;
    }

    if missing_low {
        score -= 0.2;
    }

    let staleness_penalty = (staleness_seconds / max_staleness_seconds).clamp(0.0, 1.0) * 0.3;
    score -= staleness_penalty;

    if spread_pct > 0.08 {
        score -= 0.2;
    }

    score.clamp(0.0, 1.0)
}
```

---
