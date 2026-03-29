-- Persistent API response cache for graceful degradation.
-- When external APIs fail, routes serve the last successful response from here.
-- This is a plain table (not a hypertable) — one row per API endpoint.
CREATE TABLE IF NOT EXISTS api_cache (
    key         TEXT        PRIMARY KEY,
    data        JSONB       NOT NULL,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
