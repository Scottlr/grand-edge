# 12. Strategy Model: Spread Edge

## 12.1 Purpose

Finds tax-adjusted flipping opportunities.

This is not a prediction model. It is an execution-edge model.

## 12.2 Formula

```text
target_buy = low + buy_buffer
target_sell = high - sell_buffer

gross_edge = target_sell - target_buy
tax = min(floor(target_sell * tax_rate), tax_cap)
net_edge = gross_edge - tax - slippage
roi = net_edge / target_buy
```

## 12.3 Signal

```text
BUY if:
  roi >= min_roi
  fill_probability >= min_fill_probability
  estimated_capacity >= min_quantity
  data_quality >= min_data_quality
```

## 12.4 Example

Input:

```json
{
  "low": 1050000,
  "high": 1100000,
  "buyBuffer": 5000,
  "sellBuffer": 5000,
  "taxRate": 0.02,
  "taxCap": 5000000,
  "slippage": 2000
}
```

Calculation:

```text
target_buy = 1,055,000
target_sell = 1,095,000
gross_edge = 40,000
tax = 21,900
net_edge = 16,100
roi = 0.01526
```

Output:

```json
{
  "strategyId": "spread_edge_v1",
  "side": "buy",
  "targetEntry": 1055000,
  "targetExit": 1095000,
  "expectedNetGpPerUnit": 16100,
  "expectedRoi": 0.01526
}
```

## 12.5 Rust

```rust
#[derive(Debug)]
pub struct SpreadEdgeInput {
    pub low: i64,
    pub high: i64,
    pub buy_buffer: i64,
    pub sell_buffer: i64,
    pub tax_rate: f64,
    pub tax_cap: i64,
    pub slippage: i64,
}

#[derive(Debug)]
pub struct SpreadEdgeOutput {
    pub target_entry: i64,
    pub target_exit: i64,
    pub tax: i64,
    pub net_edge: i64,
    pub roi: f64,
}

pub fn spread_edge(input: SpreadEdgeInput) -> Option<SpreadEdgeOutput> {
    let target_entry = input.low + input.buy_buffer;
    let target_exit = input.high - input.sell_buffer;

    if target_entry <= 0 || target_exit <= target_entry {
        return None;
    }

    let tax = ge_tax(target_exit, input.tax_rate, input.tax_cap);
    let net_edge = target_exit - tax - target_entry - input.slippage;
    let roi = net_edge as f64 / target_entry as f64;

    Some(SpreadEdgeOutput {
        target_entry,
        target_exit,
        tax,
        net_edge,
        roi,
    })
}
```

---

# 13. Strategy Model: Momentum

## 13.1 Purpose

Detects items with price continuation.

## 13.2 Formula

Use weighted returns and volume confirmation:

```text
momentum_score =
    w1 * return_1h
  + w2 * return_6h
  + w3 * return_24h
  + w4 * volume_z
  - w5 * spread_pct
  - w6 * volatility
```

## 13.3 Confidence

```text
confidence = sigmoid(momentum_score / scale)
```

## 13.4 Example

Input:

```json
{
  "return1h": 0.012,
  "return6h": 0.028,
  "return24h": 0.04,
  "volumeZ": 1.9,
  "spreadPct": 0.03,
  "volatility": 0.025
}
```

Weights:

```json
{
  "w1": 1.5,
  "w2": 2.0,
  "w3": 0.5,
  "w4": 0.02,
  "w5": 1.0,
  "w6": 0.8
}
```

Calculation:

```text
score = 1.5*0.012 + 2.0*0.028 + 0.5*0.04 + 0.02*1.9 - 1.0*0.03 - 0.8*0.025
score = 0.082
confidence = sigmoid(0.082 / 0.1)
confidence = 0.694
```

Output:

```json
{
  "strategyId": "momentum_v1",
  "side": "buy",
  "score": 0.082,
  "confidence": 0.694
}
```

## 13.5 Rust

```rust
pub struct MomentumInput {
    pub return_1h: f64,
    pub return_6h: f64,
    pub return_24h: f64,
    pub volume_z: f64,
    pub spread_pct: f64,
    pub volatility: f64,
}

pub fn momentum_score(input: MomentumInput) -> f64 {
    1.5 * input.return_1h
        + 2.0 * input.return_6h
        + 0.5 * input.return_24h
        + 0.02 * input.volume_z
        - 1.0 * input.spread_pct
        - 0.8 * input.volatility
}

pub fn momentum_confidence(score: f64) -> f64 {
    sigmoid(score / 0.1)
}
```

---

# 14. Strategy Model: Mean Reversion

## 14.1 Purpose

Detects items that appear temporarily underpriced or overpriced relative to recent fair value.

## 14.2 Formula

```text
z = (mid_t - rolling_mean) / rolling_std
```

Expected return to mean:

```text
expected_return = (rolling_mean - mid_t) / mid_t
```

## 14.3 Signal

```text
BUY if z <= -2.0 and liquidity is acceptable
CASHOUT if z >= 0
AVOID if volume is too low
```

## 14.4 Example

Input:

```json
{
  "mid": 90,
  "rollingMean": 100,
  "rollingStd": 5
}
```

Output:

```json
{
  "zScore": -2.0,
  "expectedReturnToMean": 0.1111,
  "side": "buy"
}
```

## 14.5 Rust

```rust
pub fn mean_reversion_signal(mid: f64, rolling_mean: f64, rolling_std: f64) -> Option<(&'static str, f64)> {
    let z = z_score(mid, rolling_mean, rolling_std)?;
    let expected_return = (rolling_mean - mid) / mid;

    let side = if z <= -2.0 {
        "buy"
    } else if z >= 0.0 {
        "cashout"
    } else {
        "hold"
    };

    Some((side, expected_return))
}
```

---
