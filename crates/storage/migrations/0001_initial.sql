CREATE TABLE items (
  item_id BIGINT PRIMARY KEY,
  name TEXT NOT NULL,
  examine TEXT,
  members BOOLEAN NOT NULL,
  buy_limit INT,
  low_alch BIGINT,
  high_alch BIGINT,
  value BIGINT,
  icon_source_file_name TEXT,
  icon_canonical_file_name TEXT,
  icon_cdn_url TEXT,
  icon_source TEXT,
  updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE latest_prices (
  item_id BIGINT NOT NULL REFERENCES items(item_id),
  high BIGINT,
  high_time TIMESTAMPTZ,
  low BIGINT,
  low_time TIMESTAMPTZ,
  observed_at TIMESTAMPTZ NOT NULL,
  PRIMARY KEY (item_id, observed_at)
);

CREATE TABLE interval_prices (
  item_id BIGINT NOT NULL REFERENCES items(item_id),
  bucket_start TIMESTAMPTZ NOT NULL,
  interval TEXT NOT NULL,
  avg_high_price BIGINT,
  high_price_volume BIGINT NOT NULL DEFAULT 0,
  avg_low_price BIGINT,
  low_price_volume BIGINT NOT NULL DEFAULT 0,
  PRIMARY KEY (item_id, interval, bucket_start)
);

CREATE TABLE features (
  item_id BIGINT NOT NULL REFERENCES items(item_id),
  as_of TIMESTAMPTZ NOT NULL,
  feature_set_version TEXT NOT NULL,
  features JSONB NOT NULL,
  PRIMARY KEY (item_id, as_of, feature_set_version)
);

CREATE TABLE strategy_predictions (
  prediction_id UUID PRIMARY KEY,
  strategy_id TEXT NOT NULL,
  model_version TEXT NOT NULL,
  item_id BIGINT NOT NULL REFERENCES items(item_id),
  as_of TIMESTAMPTZ NOT NULL,
  horizon_secs BIGINT NOT NULL,
  side TEXT NOT NULL,
  expected_return DOUBLE PRECISION NOT NULL,
  confidence DOUBLE PRECISION NOT NULL,
  expected_net_gp_per_unit BIGINT NOT NULL,
  target_entry BIGINT,
  target_exit BIGINT,
  stop_loss BIGINT,
  take_profit BIGINT,
  max_quantity BIGINT,
  explanation JSONB NOT NULL
);

CREATE TABLE users (
  user_id UUID PRIMARY KEY,
  display_name TEXT,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE recommendations (
  recommendation_id UUID PRIMARY KEY,
  user_id UUID REFERENCES users(user_id),
  item_id BIGINT NOT NULL REFERENCES items(item_id),
  as_of TIMESTAMPTZ NOT NULL,
  action TEXT NOT NULL,
  score DOUBLE PRECISION NOT NULL,
  prediction_confidence DOUBLE PRECISION,
  execution_confidence DOUBLE PRECISION,
  recommendation_confidence DOUBLE PRECISION NOT NULL,
  expected_net_gp BIGINT,
  expected_roi DOUBLE PRECISION,
  risk_label TEXT,
  reasons JSONB NOT NULL,
  explanation JSONB NOT NULL
);

CREATE TABLE user_positions (
  position_id UUID PRIMARY KEY,
  user_id UUID NOT NULL REFERENCES users(user_id),
  item_id BIGINT NOT NULL REFERENCES items(item_id),
  quantity BIGINT NOT NULL,
  avg_buy_price BIGINT NOT NULL,
  bought_at TIMESTAMPTZ,
  notes TEXT,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE simulation_runs (
  run_id UUID PRIMARY KEY,
  name TEXT NOT NULL,
  strategy_config JSONB NOT NULL,
  started_at TIMESTAMPTZ NOT NULL,
  finished_at TIMESTAMPTZ,
  status TEXT NOT NULL
);

CREATE TABLE simulated_orders (
  order_id UUID PRIMARY KEY,
  run_id UUID NOT NULL REFERENCES simulation_runs(run_id),
  strategy_id TEXT NOT NULL,
  model_version TEXT NOT NULL,
  item_id BIGINT NOT NULL REFERENCES items(item_id),
  created_at TIMESTAMPTZ NOT NULL,
  side TEXT NOT NULL,
  quantity BIGINT NOT NULL,
  limit_price BIGINT,
  status TEXT NOT NULL,
  filled_at TIMESTAMPTZ,
  fill_price BIGINT,
  realized_profit_gp BIGINT,
  explanation JSONB NOT NULL
);

CREATE TABLE paper_bets (
  bet_id UUID PRIMARY KEY,
  run_id UUID NOT NULL REFERENCES simulation_runs(run_id),
  recommendation_id UUID REFERENCES recommendations(recommendation_id),
  strategy_id TEXT NOT NULL,
  model_version TEXT NOT NULL,
  item_id BIGINT NOT NULL REFERENCES items(item_id),
  entry_time TIMESTAMPTZ NOT NULL,
  entry_price BIGINT NOT NULL,
  quantity BIGINT NOT NULL,
  target_exit BIGINT,
  stop_loss BIGINT,
  exit_time TIMESTAMPTZ,
  exit_price BIGINT,
  tax_paid BIGINT NOT NULL DEFAULT 0,
  realized_profit_gp BIGINT,
  realized_roi DOUBLE PRECISION,
  max_drawdown DOUBLE PRECISION,
  hit_reason TEXT,
  status TEXT NOT NULL,
  explanation JSONB NOT NULL
);

CREATE TABLE strategy_metrics (
  strategy_id TEXT NOT NULL,
  model_version TEXT NOT NULL,
  horizon_secs BIGINT NOT NULL,
  window_name TEXT NOT NULL,
  window_start TIMESTAMPTZ NOT NULL,
  window_end TIMESTAMPTZ NOT NULL,
  metrics JSONB NOT NULL,
  PRIMARY KEY (strategy_id, model_version, horizon_secs, window_name, window_start, window_end)
);

CREATE INDEX latest_prices_observed_at_idx ON latest_prices (observed_at DESC);
CREATE INDEX interval_prices_item_interval_time_idx ON interval_prices (item_id, interval, bucket_start DESC);
CREATE INDEX features_item_version_time_idx ON features (item_id, feature_set_version, as_of DESC);
CREATE INDEX strategy_predictions_item_time_idx ON strategy_predictions (item_id, as_of DESC);
CREATE INDEX recommendations_user_time_idx ON recommendations (user_id, as_of DESC);
CREATE INDEX user_positions_user_item_idx ON user_positions (user_id, item_id);
CREATE INDEX simulated_orders_run_status_idx ON simulated_orders (run_id, status);
CREATE INDEX paper_bets_strategy_time_idx ON paper_bets (strategy_id, model_version, entry_time DESC);
CREATE INDEX strategy_metrics_lookup_idx ON strategy_metrics (strategy_id, model_version, horizon_secs, window_end DESC);
