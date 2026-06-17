CREATE TABLE user_auth_identities (
  user_id UUID PRIMARY KEY REFERENCES users(user_id),
  email TEXT NOT NULL UNIQUE,
  password_hash TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE user_risk_profiles (
  user_id UUID PRIMARY KEY REFERENCES users(user_id),
  max_gp_per_item BIGINT NOT NULL,
  max_portfolio_drawdown DOUBLE PRECISION NOT NULL,
  min_expected_roi DOUBLE PRECISION NOT NULL,
  min_confidence DOUBLE PRECISION NOT NULL,
  participation_rate DOUBLE PRECISION NOT NULL,
  preferred_execution_mode TEXT NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE user_sessions (
  session_id UUID PRIMARY KEY,
  user_id UUID NOT NULL REFERENCES users(user_id),
  expires_at TIMESTAMPTZ NOT NULL,
  revoked_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX user_auth_identities_email_idx ON user_auth_identities (email);
CREATE INDEX user_sessions_user_id_idx ON user_sessions (user_id, expires_at DESC);
