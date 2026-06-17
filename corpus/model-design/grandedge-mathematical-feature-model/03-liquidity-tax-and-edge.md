# 7. Volume and Liquidity Features

## 7.1 Total Volume

### Purpose

Measures activity.

### Formula

```text
volume_t = high_price_volume_t + low_price_volume_t
```

### Example

Input:

```json
{
  "highPriceVolume": 120,
  "lowPriceVolume": 180
}
```

Output:

```json
{
  "volume": 300
}
```

### Rust

```rust
pub fn total_volume(high_volume: Option<f64>, low_volume: Option<f64>) -> f64 {
    high_volume.unwrap_or(0.0) + low_volume.unwrap_or(0.0)
}
```

---

## 7.2 Volume Z-Score

### Purpose

Detects unusual trading activity.

### Formula

```text
volume_z_t = (volume_t - mean_volume_n) / std_volume_n
```

### Example

Input:

```json
{
  "volume": 400,
  "rollingMeanVolume": 250,
  "rollingStdVolume": 100
}
```

Output:

```json
{
  "volumeZ": 1.5
}
```

### What it provides

Used by:

* Momentum confirmation
* Pump detection
* Fill probability
* Liquidity confidence

---

## 7.3 Fill Capacity Estimate

### Purpose

Estimates how much quantity the system can reasonably recommend.

### Formula

```text
estimated_capacity = min(buy_limit, floor(interval_volume * participation_rate))
```

Where `participation_rate` is the fraction of observed volume we assume we can capture.

### Example

Input:

```json
{
  "buyLimit": 70,
  "intervalVolume": 400,
  "participationRate": 0.05
}
```

Output:

```json
{
  "estimatedCapacity": 20
}
```

### Rust

```rust
pub fn estimated_fill_capacity(
    buy_limit: i64,
    interval_volume: f64,
    participation_rate: f64,
) -> i64 {
    let volume_capacity = (interval_volume * participation_rate).floor() as i64;

    buy_limit.min(volume_capacity).max(0)
}
```

---

## 7.4 Fill Probability

### Purpose

Approximates whether a passive order is likely to fill.

Because we do not have a full order book, this is estimated.

### Formula

```text
fill_probability = sigmoid(
    β0
  + β1 * volume_z
  - β2 * spread_pct
  - β3 * price_staleness
)
```

Where:

```text
sigmoid(x) = 1 / (1 + e^-x)
```

### Example

Input:

```json
{
  "bias": 0.1,
  "volumeZ": 1.2,
  "spreadPct": 0.02,
  "priceStalenessMinutes": 2,
  "weights": {
    "volumeZ": 0.6,
    "spreadPct": 8.0,
    "staleness": 0.05
  }
}
```

Calculation:

```text
x = 0.1 + 0.6*1.2 - 8.0*0.02 - 0.05*2
x = 0.56
sigmoid(0.56) = 0.636
```

Output:

```json
{
  "fillProbability": 0.636
}
```

### Rust

```rust
pub fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

pub fn fill_probability(
    bias: f64,
    volume_z: f64,
    spread_pct: f64,
    staleness_minutes: f64,
) -> f64 {
    let x =
        bias
        + 0.6 * volume_z
        - 8.0 * spread_pct
        - 0.05 * staleness_minutes;

    sigmoid(x)
}
```

### What it provides

Used by:

* Quantity sizing
* Recommendation confidence
* Passive execution simulation
* Avoiding low-liquidity traps

---

# 8. Tax, Slippage, and Net Edge

## 8.1 Tax

### Purpose

Models sale cost.

Tax rules should be config-driven, not hardcoded.

### Formula

```text
tax = min(floor(sell_price * tax_rate), tax_cap)
```

Optional:

```text
if sell_price < taxable_threshold:
    tax = 0
```

### Example

Input:

```json
{
  "sellPrice": 103000,
  "taxRate": 0.02,
  "taxCap": 5000000
}
```

Output:

```json
{
  "tax": 2060
}
```

### Rust

```rust
pub fn ge_tax(sell_price: i64, tax_rate: f64, tax_cap: i64) -> i64 {
    let raw_tax = (sell_price as f64 * tax_rate).floor() as i64;

    raw_tax.min(tax_cap).max(0)
}
```

---

## 8.2 Net Profit Per Unit

### Formula

```text
net_profit_per_unit = sell_price - tax - buy_price - slippage
```

### Example

Input:

```json
{
  "buyPrice": 100000,
  "sellPrice": 103000,
  "tax": 2060,
  "slippage": 0
}
```

Output:

```json
{
  "netProfitPerUnit": 940
}
```

### Rust

```rust
pub fn net_profit_per_unit(
    buy_price: i64,
    sell_price: i64,
    tax: i64,
    slippage: i64,
) -> i64 {
    sell_price - tax - buy_price - slippage
}
```

---

## 8.3 ROI

### Formula

```text
roi = net_profit_per_unit / buy_price
```

### Example

Input:

```json
{
  "netProfitPerUnit": 940,
  "buyPrice": 100000
}
```

Output:

```json
{
  "roi": 0.0094
}
```

### Rust

```rust
pub fn roi(net_profit: i64, buy_price: i64) -> Option<f64> {
    if buy_price <= 0 {
        return None;
    }

    Some(net_profit as f64 / buy_price as f64)
}
```

### What it provides

Used by:

* Recommendation score
* Portfolio allocation
* Strategy filter
* Cashout decision
* Simulation PnL

---
