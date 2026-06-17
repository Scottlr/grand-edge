# 29. Reason Atoms and Explanation Scoring

## 29.1 Purpose

Every recommendation should be explainable with measurable evidence.

## 29.2 Reason Atom

```json
{
  "type": "model_signal",
  "key": "momentum_volume_confirmed",
  "label": "Momentum is confirmed by elevated volume",
  "direction": "positive",
  "weight": 0.28,
  "evidence": {
    "return6h": 0.028,
    "volumeZ": 1.9
  }
}
```

## 29.3 Explanation Confidence

Example formula:

```text
explanation_confidence =
    supported_reason_weight / total_reason_weight
```

Where supported reasons are reasons with non-missing, fresh, valid evidence.

### Example

Input:

```json
{
  "totalReasonWeight": 1.0,
  "supportedReasonWeight": 0.83
}
```

Output:

```json
{
  "explanationConfidence": 0.83
}
```

## 29.4 Reason Outcome Tracking

Track:

```text
reason_key
sample_size
win_rate
avg_net_gp
calibration_error
```

Example output:

```json
{
  "reasonKey": "spread_survives_tax",
  "sampleSize": 1842,
  "winRate": 0.59,
  "avgNetGp": 42000,
  "calibrationError": 0.04
}
```

This allows the UI to say:

```text
This reason has historically performed well.
```

---

# 30. Feature Snapshot Contract

Every model must receive a versioned feature snapshot.

## 30.1 Example

```json
{
  "featureSnapshotId": "fs_123",
  "itemId": 4151,
  "asOf": "2026-06-16T10:00:00Z",
  "featureSetVersion": "features_v1",
  "sourceWindow": {
    "from": "2026-06-09T10:00:00Z",
    "to": "2026-06-16T10:00:00Z"
  },
  "features": {
    "mid": 1067500,
    "spreadPct": 0.03279,
    "return1h": 0.012,
    "return6h": 0.028,
    "return24h": 0.040,
    "volumeZ1h": 1.9,
    "volatility24h": 0.041,
    "priceStalenessSeconds": 57,
    "kalmanFairValue": 1072000,
    "kalmanMispricingPct": -0.0042,
    "zScore24h": -0.4,
    "fillProbability": 0.68,
    "dataQualityConfidence": 0.91
  }
}
```

## 30.2 Rule

A prediction must reference the exact feature snapshot used.

```text
prediction.feature_snapshot_id -> feature_snapshots.feature_snapshot_id
```

This enables reproducibility.

---

# 31. Prediction Output Contract

```json
{
  "predictionId": "pred_123",
  "featureSnapshotId": "fs_123",
  "itemId": 4151,
  "asOf": "2026-06-16T10:00:00Z",
  "horizonSeconds": 21600,
  "modelId": "gbm_ranker_v1",
  "modelVersion": "2026-06-16.1",
  "predictedDirection": "up",
  "predictedReturn": 0.024,
  "confidence": 0.64,
  "predictionInterval": {
    "low": 0.006,
    "high": 0.041
  },
  "reasonAtoms": [
    {
      "key": "positive_momentum",
      "weight": 0.28
    },
    {
      "key": "volume_confirmed",
      "weight": 0.18
    }
  ]
}
```

---

# 32. Recommendation Output Contract

```json
{
  "recommendationId": "rec_123",
  "itemId": 4151,
  "userId": null,
  "asOf": "2026-06-16T10:00:00Z",
  "action": "buy",
  "recommendationConfidence": 0.58,
  "expectedNetGp": 478000,
  "expectedRoi": 0.0227,
  "targetEntry": 1055000,
  "targetExit": 1090000,
  "stopLoss": 1030000,
  "quantityHint": 20,
  "riskLabel": "medium",
  "linkedPredictionIds": [
    "pred_123",
    "pred_124"
  ],
  "confidenceBreakdown": {
    "predictionConfidence": 0.64,
    "dataQualityConfidence": 0.91,
    "modelCalibrationConfidence": 0.61,
    "liquidityConfidence": 0.68,
    "explanationConfidence": 0.83
  },
  "reasonAtoms": [
    {
      "key": "spread_survives_tax",
      "direction": "positive",
      "weight": 0.22
    },
    {
      "key": "volatility_elevated",
      "direction": "negative",
      "weight": -0.11
    }
  ],
  "invalidationRules": [
    "Low price falls below 1,030,000",
    "Spread widens above 4.5%",
    "Volume falls below the 30-day median"
  ]
}
```

---

# 33. Python-to-Rust Model Artifact Contract

Python trains models.

Rust serves models.

## 33.1 Model Card

```json
{
  "modelId": "gbm_ranker_v1",
  "modelVersion": "2026-06-16.1",
  "trainedAt": "2026-06-16T08:00:00Z",
  "target": "future_net_return_6h",
  "features": [
    "spreadPct",
    "return1h",
    "return6h",
    "return24h",
    "volumeZ1h",
    "volatility24h",
    "priceStalenessSeconds",
    "kalmanMispricingPct",
    "fillProbability"
  ],
  "metrics": {
    "directionalAccuracy": 0.57,
    "brierScore": 0.218,
    "avgNetReturn": 0.011,
    "maxDrawdown": -0.064
  }
}
```

## 33.2 Rust Inference Shape

```rust
pub struct ModelInput {
    pub feature_names: Vec<String>,
    pub values: Vec<f32>,
}

pub struct ModelOutput {
    pub predicted_return: f64,
    pub probability_positive: Option<f64>,
    pub raw_score: f64,
}
```

## 33.3 Rule

Rust must validate:

```text
feature names match
feature order matches
model version is known
model card exists
calibration artifact exists
```

before serving predictions.

---
