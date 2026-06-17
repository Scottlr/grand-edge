# 3. Core Derived Price Features

## 3.1 Mid Price

### Purpose

Provides a single representative price for modelling.

### Formula

```text
mid_t = (high_t + low_t) / 2
```

If one side is missing, either:

```text
mid_t = high_t
mid_t = low_t
```

or mark the feature as degraded.

### Example

Input:

```json
{
  "high": 1085000,
  "low": 1050000
}
```

Output:

```json
{
  "mid": 1067500
}
```

### Rust

```rust
pub fn mid_price(high: Option<f64>, low: Option<f64>) -> Option<f64> {
    match (high, low) {
        (Some(h), Some(l)) => Some((h + l) / 2.0),
        (Some(h), None) => Some(h),
        (None, Some(l)) => Some(l),
        (None, None) => None,
    }
}
```

### Python

```python
def mid_price(high: float | None, low: float | None) -> float | None:
    if high is not None and low is not None:
        return (high + low) / 2
    if high is not None:
        return high
    if low is not None:
        return low
    return None
```

---

## 3.2 Absolute Spread

### Purpose

Measures gap between high-side and low-side prices.

Large spread can imply opportunity or danger.

### Formula

```text
spread_abs_t = high_t - low_t
```

### Example

Input:

```json
{
  "high": 1085000,
  "low": 1050000
}
```

Output:

```json
{
  "spreadAbs": 35000
}
```

### Rust

```rust
pub fn spread_abs(high: f64, low: f64) -> f64 {
    high - low
}
```

---

## 3.3 Spread Percentage

### Purpose

Normalises spread relative to item price.

### Formula

```text
spread_pct_t = (high_t - low_t) / mid_t
```

### Example

Input:

```json
{
  "high": 1085000,
  "low": 1050000,
  "mid": 1067500
}
```

Output:

```json
{
  "spreadPct": 0.03279
}
```

### Rust

```rust
pub fn spread_pct(high: f64, low: f64) -> Option<f64> {
    let mid = (high + low) / 2.0;

    if mid <= 0.0 {
        return None;
    }

    Some((high - low) / mid)
}
```

### What it provides

Used by:

* Spread strategy
* Liquidity risk
* Slippage estimation
* Recommendation confidence
* Avoid filters

---

# 4. Return Features

## 4.1 Simple Return

### Purpose

Measures percentage price change.

### Formula

```text
return_t = (price_t - price_{t-k}) / price_{t-k}
```

### Example

Input:

```json
{
  "priceNow": 106,
  "pricePrevious": 100
}
```

Output:

```json
{
  "simpleReturn": 0.06
}
```

### Rust

```rust
pub fn simple_return(current: f64, previous: f64) -> Option<f64> {
    if previous <= 0.0 {
        return None;
    }

    Some((current - previous) / previous)
}
```

---

## 4.2 Log Return

### Purpose

Better for statistical modelling because log returns are additive over time.

### Formula

```text
log_return_t = ln(price_t / price_{t-k})
```

### Example

Input:

```json
{
  "priceNow": 106,
  "pricePrevious": 100
}
```

Output:

```json
{
  "logReturn": 0.05827
}
```

### Rust

```rust
pub fn log_return(current: f64, previous: f64) -> Option<f64> {
    if current <= 0.0 || previous <= 0.0 {
        return None;
    }

    Some((current / previous).ln())
}
```

### Python

```python
import math

def log_return(current: float, previous: float) -> float | None:
    if current <= 0 or previous <= 0:
        return None

    return math.log(current / previous)
```

### What it provides

Used by:

* Momentum
* Volatility
* ARIMA-style models
* GARCH-style models
* Regime detection
* Gradient boosted rankers

---
