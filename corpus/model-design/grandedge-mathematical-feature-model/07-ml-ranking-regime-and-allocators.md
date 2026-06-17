# 18. Gradient-Boosted Ranking Model

## 18.1 Purpose

Ranks items by expected opportunity quality.

This is likely the strongest practical ML model for the project once enough historical data exists.

## 18.2 Features

Example feature vector:

```json
{
  "spreadPct": 0.032,
  "return1h": 0.012,
  "return6h": 0.028,
  "return24h": 0.040,
  "volumeZ1h": 1.9,
  "volatility24h": 0.041,
  "priceStalenessSeconds": 57,
  "kalmanMispricingPct": -0.004,
  "zScore24h": -0.4,
  "fillProbability": 0.68
}
```

## 18.3 Target Options

Regression target:

```text
future_net_return_6h
```

Classification target:

```text
1 if future_net_return_6h > min_required_return else 0
```

Ranking target:

```text
rank items by realised future opportunity quality
```

## 18.4 Loss Concept

For regression:

```text
loss = mean((actual_return - predicted_return)^2)
```

For classification:

```text
log_loss = -[y log(p) + (1-y) log(1-p)]
```

## 18.5 Python Training Example

```python
import lightgbm as lgb
import numpy as np

X = np.array([
    [0.032, 0.012, 0.028, 1.9, 0.041],
    [0.010, -0.004, -0.012, -0.5, 0.018],
    [0.045, 0.018, 0.034, 2.2, 0.055],
], dtype=float)

y = np.array([0.021, -0.006, 0.018], dtype=float)

model = lgb.LGBMRegressor(
    objective="regression",
    n_estimators=50,
    learning_rate=0.05,
    max_depth=4,
)

model.fit(X, y)

prediction = model.predict(np.array([[0.032, 0.012, 0.028, 1.9, 0.041]]))[0]

print({
    "predicted_return": float(prediction)
})
```

## 18.6 Expected Output Shape

```json
{
  "modelId": "gbm_ranker_v1",
  "predictedReturn6h": 0.019,
  "rankScore": 0.014,
  "probabilityPositive": 0.64
}
```

## 18.7 What it provides

Used by:

* Opportunity ranking
* Recommendation scoring
* Ensemble weighting
* Portfolio allocation
* Model accuracy dashboard

---

# 19. Regime Detection Model

## 19.1 Purpose

Detects market state so strategies can adapt.

Possible regimes:

```text
calm_liquid
trending_up
trending_down
volatile
illiquid
```

## 19.2 Simple Rule-Based Regime

Before HMM/ML, start with explicit thresholds.

### Features

```text
return_6h
volatility_24h
spread_pct
volume_z
```

### Example rule

```text
if spread_pct > 0.06 and volume_z < -1:
    regime = illiquid
else if volatility_24h > 0.05:
    regime = volatile
else if return_6h > 0.025 and volume_z > 1:
    regime = trending_up
else if return_6h < -0.025:
    regime = trending_down
else:
    regime = calm_liquid
```

### Rust

```rust
pub fn rule_based_regime(
    return_6h: f64,
    volatility_24h: f64,
    spread_pct: f64,
    volume_z: f64,
) -> &'static str {
    if spread_pct > 0.06 && volume_z < -1.0 {
        return "illiquid";
    }

    if volatility_24h > 0.05 {
        return "volatile";
    }

    if return_6h > 0.025 && volume_z > 1.0 {
        return "trending_up";
    }

    if return_6h < -0.025 {
        return "trending_down";
    }

    "calm_liquid"
}
```

## 19.3 HMM Concept

A hidden Markov model assumes:

```text
hidden state_t -> observed features_t
hidden state_t depends on hidden state_{t-1}
```

Transition matrix:

```text
A[i, j] = P(state_t = j | state_{t-1} = i)
```

Emission model:

```text
P(observation_t | state_t)
```

## 19.4 What it provides

Used by:

* Strategy weighting
* Confidence reduction
* Model filtering
* UI market regime label
* Risk control

---

# 20. Contextual Bandit Strategy Allocator

## 20.1 Purpose

Learns which strategy to trust in the current context.

Each strategy is an arm:

```text
spread_edge
momentum
mean_reversion
kalman
gbm_ranker
```

Context:

```text
current item features + market regime + recent strategy accuracy
```

Reward:

```text
realised_net_return
```

## 20.2 LinUCB Formula

For each arm `a`:

```text
score_a = θ_a^T x + α * sqrt(x^T A_a^-1 x)
```

Where:

* `θ_a` is estimated reward weights
* `x` is context vector
* `α` controls exploration
* `A_a` tracks uncertainty

## 20.3 Simplified UCB Example

```text
score = predicted_reward + exploration_bonus
```

### Example

Input:

```json
{
  "strategy": "momentum_v1",
  "predictedReward": 0.012,
  "uncertainty": 0.006,
  "alpha": 1.5
}
```

Output:

```json
{
  "ucbScore": 0.021
}
```

### Rust

```rust
pub fn ucb_score(predicted_reward: f64, uncertainty: f64, alpha: f64) -> f64 {
    predicted_reward + alpha * uncertainty
}
```

## 20.4 What it provides

Used by:

* Strategy selection
* Exploration
* Avoiding stale best-strategy assumptions
* Adaptive recommendation weighting

---

# 21. Online Ensemble Weighting

## 21.1 Purpose

Combines model predictions and adapts to recent performance.

## 21.2 Hedge / Multiplicative Weights

### Formula

```text
w_i,new = w_i,old * exp(-η * loss_i)
```

Then normalise:

```text
w_i = w_i / Σw
```

Where:

* `η` is learning rate
* `loss_i` is recent model loss

## 21.3 Example

Input:

```json
{
  "eta": 10,
  "weights": [0.5, 0.5],
  "losses": [0.01, 0.04]
}
```

Calculation:

```text
w1 = 0.5 * exp(-10 * 0.01) = 0.4524
w2 = 0.5 * exp(-10 * 0.04) = 0.3352
normalised w1 = 0.574
normalised w2 = 0.426
```

Output:

```json
{
  "weightsAfter": [0.574, 0.426]
}
```

## 21.4 Rust

```rust
pub fn hedge_update(weights: &[f64], losses: &[f64], eta: f64) -> Option<Vec<f64>> {
    if weights.len() != losses.len() || weights.is_empty() {
        return None;
    }

    let mut updated = Vec::with_capacity(weights.len());

    for (weight, loss) in weights.iter().zip(losses.iter()) {
        updated.push(weight * (-eta * loss).exp());
    }

    let total = updated.iter().sum::<f64>();

    if total <= 0.0 {
        return None;
    }

    Some(updated.into_iter().map(|w| w / total).collect())
}
```

## 21.5 What it provides

Used by:

* Model ensemble
* Strategy trust weighting
* Performance-adaptive recommendations

---
