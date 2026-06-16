CREATE TABLE feature_snapshots (
  feature_snapshot_id UUID PRIMARY KEY,
  item_id BIGINT NOT NULL REFERENCES items(item_id),
  as_of TIMESTAMPTZ NOT NULL,
  feature_set_version TEXT NOT NULL,
  graph_version TEXT,
  source_window_start TIMESTAMPTZ NOT NULL,
  source_window_end TIMESTAMPTZ NOT NULL,
  features JSONB NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  UNIQUE (item_id, as_of, feature_set_version)
);

CREATE TABLE predictions (
  prediction_id UUID PRIMARY KEY,
  feature_snapshot_id UUID NOT NULL REFERENCES feature_snapshots(feature_snapshot_id),
  item_id BIGINT NOT NULL REFERENCES items(item_id),
  as_of TIMESTAMPTZ NOT NULL,
  horizon_secs BIGINT NOT NULL,
  model_id TEXT NOT NULL,
  model_version TEXT NOT NULL,
  predicted_direction TEXT NOT NULL,
  predicted_return DOUBLE PRECISION,
  confidence DOUBLE PRECISION NOT NULL,
  prediction_interval_low DOUBLE PRECISION,
  prediction_interval_high DOUBLE PRECISION,
  explanation JSONB NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE recommendation_prediction_links (
  recommendation_id UUID NOT NULL REFERENCES recommendations(recommendation_id) ON DELETE CASCADE,
  prediction_id UUID NOT NULL REFERENCES predictions(prediction_id),
  contribution_weight DOUBLE PRECISION NOT NULL,
  PRIMARY KEY (recommendation_id, prediction_id)
);

CREATE TABLE recommendation_outcomes (
  recommendation_id UUID PRIMARY KEY REFERENCES recommendations(recommendation_id) ON DELETE CASCADE,
  evaluated_at TIMESTAMPTZ NOT NULL,
  horizon_secs BIGINT NOT NULL,
  actual_return DOUBLE PRECISION,
  actual_net_gp BIGINT,
  direction_correct BOOLEAN,
  hit_take_profit BOOLEAN NOT NULL,
  hit_stop_loss BOOLEAN NOT NULL,
  max_favourable_excursion DOUBLE PRECISION,
  max_adverse_excursion DOUBLE PRECISION,
  outcome_label TEXT NOT NULL
);

CREATE TABLE reason_outcomes (
  reason_type TEXT NOT NULL,
  reason_key TEXT NOT NULL,
  model_version TEXT NOT NULL,
  window_start TIMESTAMPTZ NOT NULL,
  window_end TIMESTAMPTZ NOT NULL,
  sample_size INT NOT NULL,
  win_rate DOUBLE PRECISION,
  avg_actual_return DOUBLE PRECISION,
  avg_net_gp BIGINT,
  calibration_error DOUBLE PRECISION,
  PRIMARY KEY (reason_type, reason_key, model_version, window_start, window_end)
);

CREATE INDEX feature_snapshots_item_time_idx ON feature_snapshots (item_id, as_of DESC);
CREATE INDEX predictions_model_time_idx ON predictions (model_id, model_version, as_of DESC);
CREATE INDEX predictions_feature_snapshot_idx ON predictions (feature_snapshot_id);
CREATE INDEX recommendation_prediction_links_prediction_idx ON recommendation_prediction_links (prediction_id);
CREATE INDEX recommendation_outcomes_evaluated_idx ON recommendation_outcomes (evaluated_at DESC);
CREATE INDEX reason_outcomes_lookup_idx ON reason_outcomes (reason_type, reason_key, model_version, window_end DESC);
