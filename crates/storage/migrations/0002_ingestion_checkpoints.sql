CREATE TABLE ingestion_checkpoints (
  checkpoint_key TEXT PRIMARY KEY,
  checkpoint_value JSONB NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL
);
