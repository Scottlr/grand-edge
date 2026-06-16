ALTER TABLE reason_outcomes
ADD COLUMN recommendation_action TEXT NOT NULL DEFAULT 'watch',
ADD COLUMN execution_mode TEXT NOT NULL DEFAULT '',
ADD COLUMN confidence_bucket TEXT NOT NULL DEFAULT '',
ADD COLUMN publishable BOOLEAN NOT NULL DEFAULT FALSE;

ALTER TABLE reason_outcomes
DROP CONSTRAINT reason_outcomes_pkey;

ALTER TABLE reason_outcomes
ADD CONSTRAINT reason_outcomes_pkey PRIMARY KEY (
  reason_type,
  reason_key,
  model_version,
  recommendation_action,
  execution_mode,
  confidence_bucket,
  window_start,
  window_end
);

DROP INDEX reason_outcomes_lookup_idx;

CREATE INDEX reason_outcomes_lookup_idx
ON reason_outcomes (
  reason_type,
  reason_key,
  model_version,
  recommendation_action,
  window_end DESC
);
