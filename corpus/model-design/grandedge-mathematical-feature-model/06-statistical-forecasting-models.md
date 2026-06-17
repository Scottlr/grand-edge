# 15. Kalman Fair Value Model

## 15.1 Purpose

Estimates latent fair value from noisy observed prices.

## 15.2 Model

State equation:

```text
x_t = x_{t-1} + w_t
```

Observation equation:

```text
y_t = x_t + v_t
```

Where:

* `x_t` is hidden fair value
* `y_t` is observed mid price
* `w_t` is process noise
* `v_t` is observation noise

## 15.3 Update equations

Prediction:

```text
x_pred = x_prev
P_pred = P_prev + Q
```

Kalman gain:

```text
K = P_pred / (P_pred + R)
```

Update:

```text
x_new = x_pred + K * (observed - x_pred)
P_new = (1 - K) * P_pred
```

## 15.4 Example

Input:

```json
{
  "previousFairValue": 100,
  "previousVariance": 4,
  "processNoise": 0,
  "observationNoise": 1,
  "observedPrice": 103
}
```

Calculation:

```text
P_pred = 4
K = 4 / (4 + 1) = 0.8
x_new = 100 + 0.8 * (103 - 100) = 102.4
P_new = 0.8
```

Output:

```json
{
  "fairValue": 102.4,
  "kalmanGain": 0.8,
  "variance": 0.8
}
```

## 15.5 Rust

```rust
#[derive(Debug, Clone, Copy)]
pub struct KalmanState {
    pub estimate: f64,
    pub variance: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct KalmanUpdate {
    pub state: KalmanState,
    pub gain: f64,
}

pub fn kalman_update(
    previous: KalmanState,
    observed: f64,
    process_noise: f64,
    observation_noise: f64,
) -> KalmanUpdate {
    let predicted_estimate = previous.estimate;
    let predicted_variance = previous.variance + process_noise;

    let gain = predicted_variance / (predicted_variance + observation_noise);

    let estimate = predicted_estimate + gain * (observed - predicted_estimate);
    let variance = (1.0 - gain) * predicted_variance;

    KalmanUpdate {
        state: KalmanState { estimate, variance },
        gain,
    }
}
```

## 15.6 What it provides

Features:

```text
kalman_fair_value
kalman_mispricing = mid - fair_value
kalman_mispricing_pct = (mid - fair_value) / fair_value
kalman_uncertainty = sqrt(variance)
```

Used by:

* Fair value model
* Mean reversion
* Confidence weighting
* Anomaly detection

---

# 16. AR and ARIMA-Style Forecasting

## 16.1 Purpose

Provides a simple time-series forecast baseline.

Start with AR(1). Use full ARIMA in Python research.

## 16.2 AR(1) Formula

```text
r_t = c + φ * r_{t-1} + ε_t
```

Forecast:

```text
r_hat_{t+1} = c + φ * r_t
price_hat = price_t * exp(r_hat)
```

## 16.3 Example

Input:

```json
{
  "currentPrice": 103,
  "lastLogReturn": 0.02,
  "c": 0.001,
  "phi": 0.5
}
```

Calculation:

```text
forecast_return = 0.001 + 0.5 * 0.02 = 0.011
forecast_price = 103 * exp(0.011) = 104.139
```

Output:

```json
{
  "forecastReturn": 0.011,
  "forecastPrice": 104.139
}
```

## 16.4 Rust

```rust
pub fn ar1_forecast_return(c: f64, phi: f64, last_return: f64) -> f64 {
    c + phi * last_return
}

pub fn forecast_price_from_log_return(current_price: f64, forecast_return: f64) -> f64 {
    current_price * forecast_return.exp()
}
```

## 16.5 Python ARIMA

```python
import numpy as np
from statsmodels.tsa.arima.model import ARIMA

prices = np.array([100, 101, 102, 101, 103, 104], dtype=float)
log_returns = np.diff(np.log(prices))

model = ARIMA(log_returns, order=(1, 0, 0))
fit = model.fit()

forecast_return = fit.forecast(steps=1)[0]
forecast_price = prices[-1] * np.exp(forecast_return)

print({
    "forecast_return": float(forecast_return),
    "forecast_price": float(forecast_price),
})
```

## 16.6 What it provides

Outputs:

```text
forecast_return
forecast_price
forecast_error
baseline_direction
```

Used by:

* Baseline model comparison
* Ensemble input
* Model sanity check

---

# 17. Logistic Direction Classifier

## 17.1 Purpose

Predicts probability that future return is positive.

## 17.2 Formula

```text
p(up) = sigmoid(β0 + β1x1 + β2x2 + ... + βnxn)
```

Features may include:

```text
return_1h
return_6h
spread_pct
volume_z
volatility
z_score
```

## 17.3 Example

Input:

```json
{
  "features": {
    "return1h": 0.012,
    "return6h": 0.028,
    "spreadPct": 0.03,
    "volumeZ": 1.9
  },
  "weights": {
    "bias": -0.2,
    "return1h": 4.0,
    "return6h": 3.0,
    "spreadPct": -2.0,
    "volumeZ": 0.1
  }
}
```

Calculation:

```text
x = -0.2 + 4*0.012 + 3*0.028 - 2*0.03 + 0.1*1.9
x = 0.062
p = sigmoid(0.062) = 0.5155
```

Output:

```json
{
  "probabilityUp": 0.5155,
  "direction": "up",
  "confidence": 0.5155
}
```

## 17.4 Rust

```rust
pub fn logistic_direction_probability(
    bias: f64,
    features: &[f64],
    weights: &[f64],
) -> Option<f64> {
    if features.len() != weights.len() {
        return None;
    }

    let linear = features
        .iter()
        .zip(weights.iter())
        .fold(bias, |acc, (x, w)| acc + x * w);

    Some(sigmoid(linear))
}
```

## 17.5 Python Training

```python
import numpy as np
from sklearn.linear_model import LogisticRegression

X = np.array([
    [0.012, 0.028, 0.030, 1.9],
    [-0.010, -0.022, 0.045, -0.4],
    [0.005, 0.018, 0.020, 0.8],
])

y = np.array([1, 0, 1])

model = LogisticRegression()
model.fit(X, y)

prediction = model.predict_proba([[0.012, 0.028, 0.030, 1.9]])[0, 1]

print({
    "probability_up": float(prediction)
})
```

## 17.6 What it provides

Used by:

* Prediction confidence
* Ensemble input
* Calibration layer
* Directional accuracy tracking

---
