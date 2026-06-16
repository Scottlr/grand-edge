CREATE TABLE graph_versions (
  graph_version TEXT PRIMARY KEY,
  source_hash TEXT NOT NULL,
  description TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE item_graph_nodes (
  graph_version TEXT NOT NULL REFERENCES graph_versions(graph_version) ON DELETE CASCADE,
  item_id BIGINT NOT NULL REFERENCES items(item_id),
  category TEXT,
  metadata JSONB NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  PRIMARY KEY (graph_version, item_id)
);

CREATE TABLE item_edges (
  edge_id UUID PRIMARY KEY,
  graph_version TEXT NOT NULL REFERENCES graph_versions(graph_version) ON DELETE CASCADE,
  from_item_id BIGINT NOT NULL REFERENCES items(item_id),
  to_item_id BIGINT NOT NULL REFERENCES items(item_id),
  edge_type TEXT NOT NULL,
  direction TEXT NOT NULL,
  sign DOUBLE PRECISION NOT NULL,
  weight DOUBLE PRECISION NOT NULL,
  lag_seconds INT,
  confidence DOUBLE PRECISION NOT NULL,
  source_type TEXT NOT NULL,
  source_ref TEXT,
  formula JSONB NOT NULL,
  requires_review BOOLEAN NOT NULL DEFAULT false,
  active BOOLEAN NOT NULL DEFAULT true,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  CHECK (weight >= 0.0 AND weight <= 1.0),
  CHECK (confidence >= 0.0 AND confidence <= 1.0),
  CHECK (sign IN (-1.0, 0.0, 1.0))
);

CREATE TABLE edge_observations (
  edge_id UUID NOT NULL REFERENCES item_edges(edge_id) ON DELETE CASCADE,
  observed_at TIMESTAMPTZ NOT NULL,
  method TEXT NOT NULL,
  window_start TIMESTAMPTZ NOT NULL,
  window_end TIMESTAMPTZ NOT NULL,
  statistic DOUBLE PRECISION,
  p_value DOUBLE PRECISION,
  estimated_lag_seconds INT,
  estimated_effect DOUBLE PRECISION,
  confidence DOUBLE PRECISION NOT NULL,
  metadata JSONB NOT NULL,
  PRIMARY KEY (edge_id, observed_at, method),
  CHECK (confidence >= 0.0 AND confidence <= 1.0)
);

CREATE TABLE market_events (
  event_id UUID PRIMARY KEY,
  graph_version TEXT NOT NULL REFERENCES graph_versions(graph_version) ON DELETE CASCADE,
  event_type TEXT NOT NULL,
  title TEXT NOT NULL,
  occurred_at TIMESTAMPTZ NOT NULL,
  source_ref TEXT NOT NULL,
  metadata JSONB NOT NULL
);

CREATE TABLE market_event_items (
  event_id UUID NOT NULL REFERENCES market_events(event_id) ON DELETE CASCADE,
  item_id BIGINT NOT NULL REFERENCES items(item_id),
  relation TEXT NOT NULL,
  confidence DOUBLE PRECISION NOT NULL,
  PRIMARY KEY (event_id, item_id, relation),
  CHECK (confidence >= 0.0 AND confidence <= 1.0)
);

CREATE TABLE corpus_sources (
  source_id TEXT PRIMARY KEY,
  source_type TEXT NOT NULL,
  title TEXT NOT NULL,
  url TEXT,
  retrieved_at TIMESTAMPTZ,
  license_note TEXT NOT NULL,
  content_hash TEXT NOT NULL,
  metadata JSONB NOT NULL
);

CREATE TABLE recommendation_graph_links (
  link_id UUID PRIMARY KEY,
  recommendation_id UUID NOT NULL REFERENCES recommendations(recommendation_id) ON DELETE CASCADE,
  graph_version TEXT NOT NULL REFERENCES graph_versions(graph_version) ON DELETE CASCADE,
  edge_id UUID REFERENCES item_edges(edge_id) ON DELETE CASCADE,
  event_id UUID REFERENCES market_events(event_id) ON DELETE CASCADE,
  contribution_weight DOUBLE PRECISION,
  explanation JSONB NOT NULL,
  CHECK (edge_id IS NOT NULL OR event_id IS NOT NULL)
);

CREATE UNIQUE INDEX recommendation_graph_links_edge_unique_idx
  ON recommendation_graph_links (recommendation_id, graph_version, edge_id)
  WHERE edge_id IS NOT NULL;

CREATE UNIQUE INDEX recommendation_graph_links_event_unique_idx
  ON recommendation_graph_links (recommendation_id, graph_version, event_id)
  WHERE event_id IS NOT NULL;

CREATE INDEX item_edges_from_idx ON item_edges (graph_version, from_item_id, active);
CREATE INDEX item_edges_to_idx ON item_edges (graph_version, to_item_id, active);
CREATE INDEX item_edges_type_idx ON item_edges (graph_version, edge_type, source_type);
CREATE INDEX edge_observations_lookup_idx ON edge_observations (edge_id, method, observed_at DESC);
CREATE INDEX market_events_time_idx ON market_events (occurred_at DESC, event_type);
