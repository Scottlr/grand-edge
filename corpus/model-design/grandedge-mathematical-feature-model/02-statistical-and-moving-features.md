# 5. Rolling Statistical Features

## 5.1 Rolling Mean

### Purpose

Estimates recent fair value.

### Formula

```text
mean_n = (x_1 + x_2 + ... + x_n) / n
```

### Example

Input:

```json
{
  "prices": [100, 102, 101, 103]
}
```

Output:

```json
{
  "rollingMean": 101.5
}
```

### Rust

```rust
pub fn mean(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }

    Some(values.iter().sum::<f64>() / values.len() as f64)
}
```

---

## 5.2 Rolling Standard Deviation

### Purpose

Measures recent price dispersion.

### Formula

```text
std_n = sqrt(Σ(x_i - mean)^2 / n)
```

Use population standard deviation for deterministic feature calculation.

### Example

Input:

```json
{
  "values": [100, 102, 101, 103]
}
```

Output:

```json
{
  "mean": 101.5,
  "std": 1.1180
}
```

### Rust

```rust
pub fn std_population(values: &[f64]) -> Option<f64> {
    let mean = mean(values)?;

    let variance = values
        .iter()
        .map(|x| {
            let diff = x - mean;
            diff * diff
        })
        .sum::<f64>() / values.len() as f64;

    Some(variance.sqrt())
}
```

---

## 5.3 Z-Score

### Purpose

Measures how far current price is from recent normal behaviour.

### Formula

```text
z_t = (x_t - mean_n) / std_n
```

### Example

Input:

```json
{
  "current": 90,
  "rollingMean": 100,
  "rollingStd": 5
}
```

Output:

```json
{
  "zScore": -2.0
}
```

### Rust

```rust
pub fn z_score(current: f64, rolling_mean: f64, rolling_std: f64) -> Option<f64> {
    if rolling_std <= 0.0 {
        return None;
    }

    Some((current - rolling_mean) / rolling_std)
}
```

### What it provides

Used by:

* Mean reversion
* Anomaly detection
* Regime detection
* Risk control
* Confidence reduction when price is extreme

---

# 6. Exponential Moving Features

## 6.1 Exponential Moving Average

### Purpose

Smooths noisy prices while weighting recent observations more strongly.

### Formula

```text
EMA_t = α * x_t + (1 - α) * EMA_{t-1}
```

Where:

```text
α = 2 / (n + 1)
```

### Example

Input:

```json
{
  "previousEma": 100,
  "current": 106,
  "alpha": 0.2
}
```

Output:

```json
{
  "ema": 101.2
}
```

### Rust

```rust
pub fn ema_step(previous_ema: f64, current: f64, alpha: f64) -> f64 {
    alpha * current + (1.0 - alpha) * previous_ema
}
```

### Python

```python
def ema_step(previous_ema: float, current: float, alpha: float) -> float:
    return alpha * current + (1 - alpha) * previous_ema
```

---

## 6.2 MACD-Style Momentum

### Purpose

Compares fast and slow EMAs to detect trend shifts.

### Formula

```text
macd_t = EMA_fast_t - EMA_slow_t
signal_t = EMA(macd_t)
histogram_t = macd_t - signal_t
```

### Example

Input:

```json
{
  "emaFast": 106,
  "emaSlow": 100,
  "signal": 3
}
```

Output:

```json
{
  "macd": 6,
  "histogram": 3
}
```

### Rust

```rust
pub fn macd(ema_fast: f64, ema_slow: f64, signal: f64) -> (f64, f64) {
    let macd = ema_fast - ema_slow;
    let histogram = macd - signal;

    (macd, histogram)
}
```

### What it provides

Used by:

* Momentum model
* Trend confirmation
* Cashout weakening signal

---
