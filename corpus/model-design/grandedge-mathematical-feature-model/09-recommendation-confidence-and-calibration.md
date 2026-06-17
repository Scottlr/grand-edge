# 24. Recommendation Scoring

## 24.1 Purpose

Turns model predictions into user-facing actions.

## 24.2 Formula

```text
score =
    expected_net_return
  + model_agreement_bonus
  + calibration_bonus
  + liquidity_bonus
  - volatility_penalty
  - spread_penalty
  - staleness_penalty
  - concentration_penalty
```

## 24.3 Example

Input:

```json
{
  "expectedNetReturn": 0.024,
  "modelAgreementBonus": 0.006,
  "calibrationBonus": 0.004,
  "liquidityBonus": 0.003,
  "volatilityPenalty": 0.008,
  "spreadPenalty": 0.004,
  "stalenessPenalty": 0.001,
  "concentrationPenalty": 0.000
}
```

Output:

```json
{
  "recommendationScore": 0.024
}
```

## 24.4 Rust

```rust
pub struct RecommendationScoreInput {
    pub expected_net_return: f64,
    pub model_agreement_bonus: f64,
    pub calibration_bonus: f64,
    pub liquidity_bonus: f64,
    pub volatility_penalty: f64,
    pub spread_penalty: f64,
    pub staleness_penalty: f64,
    pub concentration_penalty: f64,
}

pub fn recommendation_score(input: RecommendationScoreInput) -> f64 {
    input.expected_net_return
        + input.model_agreement_bonus
        + input.calibration_bonus
        + input.liquidity_bonus
        - input.volatility_penalty
        - input.spread_penalty
        - input.staleness_penalty
        - input.concentration_penalty
}
```

## 24.5 Action Mapping

```text
BUY if score >= buy_threshold and user has no overexposure
ADD if score >= add_threshold and concentration acceptable
HOLD if position exists and score remains positive
CASHOUT if net profit is available and future score has deteriorated
AVOID if score < avoid_threshold or hard blocker exists
WATCH if signal is promising but not actionable
```

---

# 25. Confidence System

## 25.1 Do Not Use One Confidence

Use separate confidence dimensions:

```json
{
  "predictionConfidence": 0.64,
  "recommendationConfidence": 0.58,
  "dataQualityConfidence": 0.91,
  "modelCalibrationConfidence": 0.61,
  "liquidityConfidence": 0.68,
  "explanationConfidence": 0.83
}
```

## 25.2 Recommendation Confidence Formula

Example:

```text
recommendation_confidence =
    0.35 * prediction_confidence
  + 0.20 * model_calibration_confidence
  + 0.20 * liquidity_confidence
  + 0.15 * data_quality_confidence
  + 0.10 * model_agreement
```

Then apply penalties:

```text
if spread_pct > max_spread:
    confidence *= 0.75

if staleness_seconds > max_staleness:
    confidence *= 0.50

if regime == volatile:
    confidence *= 0.85
```

## 25.3 Example

Input:

```json
{
  "predictionConfidence": 0.64,
  "modelCalibrationConfidence": 0.61,
  "liquidityConfidence": 0.68,
  "dataQualityConfidence": 0.91,
  "modelAgreement": 0.70
}
```

Calculation:

```text
0.35*0.64 + 0.20*0.61 + 0.20*0.68 + 0.15*0.91 + 0.10*0.70
= 0.6885
```

Output:

```json
{
  "recommendationConfidence": 0.6885
}
```

## 25.4 Rust

```rust
pub fn recommendation_confidence(
    prediction_confidence: f64,
    calibration_confidence: f64,
    liquidity_confidence: f64,
    data_quality_confidence: f64,
    model_agreement: f64,
) -> f64 {
    let raw =
        0.35 * prediction_confidence
        + 0.20 * calibration_confidence
        + 0.20 * liquidity_confidence
        + 0.15 * data_quality_confidence
        + 0.10 * model_agreement;

    raw.clamp(0.0, 1.0)
}
```

---

# 26. Probability Calibration and Brier Score

## 26.1 Purpose

Checks whether confidence means anything.

If the system says 70% confidence, similar past predictions should succeed around 70% of the time.

## 26.2 Brier Score

For binary outcome:

```text
brier = mean((predicted_probability - actual_outcome)^2)
```

Where:

```text
actual_outcome = 1 for success
actual_outcome = 0 for failure
```

## 26.3 Example

Input:

```json
{
  "predictions": [0.7, 0.8, 0.4],
  "actual": [1, 1, 0]
}
```

Calculation:

```text
((0.7 - 1)^2 + (0.8 - 1)^2 + (0.4 - 0)^2) / 3
= (0.09 + 0.04 + 0.16) / 3
= 0.0967
```

Output:

```json
{
  "brierScore": 0.0967
}
```

## 26.4 Rust

```rust
pub fn brier_score(predictions: &[f64], actuals: &[bool]) -> Option<f64> {
    if predictions.len() != actuals.len() || predictions.is_empty() {
        return None;
    }

    let total = predictions
        .iter()
        .zip(actuals.iter())
        .map(|(p, actual)| {
            let y = if *actual { 1.0 } else { 0.0 };
            let diff = p - y;
            diff * diff
        })
        .sum::<f64>();

    Some(total / predictions.len() as f64)
}
```

## 26.5 Calibration Buckets

Bucket predictions:

```text
50-60%
60-70%
70-80%
80-90%
```

For each bucket:

```text
actual_success_rate = successes / sample_size
calibration_error = |mean_predicted_confidence - actual_success_rate|
```

### Example

Input:

```json
{
  "bucket": "60-70%",
  "meanPredictedConfidence": 0.65,
  "actualSuccessRate": 0.61
}
```

Output:

```json
{
  "calibrationError": 0.04,
  "status": "well_calibrated"
}
```

---
