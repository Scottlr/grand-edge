# GrandEdge Mathematical Feature and Model Design Document

## 1. Purpose

This document defines the mathematical foundation for the GrandEdge recommendation and prediction system.

GrandEdge ingests OSRS item price data and produces:

* Features
* Predictions
* Recommendations
* Confidence metadata
* Explanation metadata
* Simulation outcomes
* Model performance metrics

The system must keep these concepts separate:

```text
market data -> feature snapshot -> model prediction -> recommendation -> explanation -> outcome evaluation
```

A prediction asks:

```text
What is likely to happen to the item price?
```

A recommendation asks:

```text
Given prediction, tax, spread, liquidity, user holdings, and risk, what should the user do?
```

Do not conflate these.

---

# 2. Core Market Data

## 2.1 Input Candle

The OSRS Wiki real-time price API gives high-side and low-side price data. For interval data, we usually have:

```json
{
  "itemId": 4151,
  "timestamp": 1781455500,
  "avgHighPrice": 1085000,
  "highPriceVolume": 120,
  "avgLowPrice": 1050000,
  "lowPriceVolume": 180
}
```

Internal representation:

```rust
#[derive(Debug, Clone)]
pub struct PriceCandle {
    pub item_id: i64,
    pub timestamp: i64,
    pub avg_high_price: Option<f64>,
    pub high_price_volume: Option<f64>,
    pub avg_low_price: Option<f64>,
    pub low_price_volume: Option<f64>,
}
```

Use `Option<f64>` because some items may have missing high/low data.

---
