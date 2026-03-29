-- Enable TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- ─────────────────────────────────────────────────────────────
-- Table 1: Hourly electricity prices
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS price_hourly (
    ts              TIMESTAMPTZ      NOT NULL,
    source          TEXT             NOT NULL,
    product         TEXT             NOT NULL,
    value_eur_mwh   DOUBLE PRECISION,
    value_pln_mwh   DOUBLE PRECISION,
    is_forecast     BOOLEAN          NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    UNIQUE (ts, source, product)
);

SELECT create_hypertable(
    'price_hourly', 'ts',
    chunk_time_interval => INTERVAL '1 week',
    if_not_exists => TRUE
);

-- ─────────────────────────────────────────────────────────────
-- Table 2: Daily fuel prices
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS fuel_daily (
    ts              TIMESTAMPTZ      NOT NULL,
    ticker          TEXT             NOT NULL,
    close           DOUBLE PRECISION NOT NULL,
    unit            TEXT             NOT NULL,
    source          TEXT             NOT NULL,
    created_at      TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    UNIQUE (ts, ticker)
);

SELECT create_hypertable(
    'fuel_daily', 'ts',
    chunk_time_interval => INTERVAL '1 month',
    if_not_exists => TRUE
);

-- ─────────────────────────────────────────────────────────────
-- Table 3: Hourly weather observations and forecasts
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS weather_hourly (
    ts              TIMESTAMPTZ      NOT NULL,
    location        TEXT             NOT NULL,
    role            TEXT             NOT NULL,
    lat             DOUBLE PRECISION NOT NULL,
    lon             DOUBLE PRECISION NOT NULL,
    windspeed_100m  DOUBLE PRECISION,
    windspeed_10m   DOUBLE PRECISION,
    temperature_2m  DOUBLE PRECISION,
    direct_rad      DOUBLE PRECISION,
    diffuse_rad     DOUBLE PRECISION,
    cloudcover      DOUBLE PRECISION,
    is_forecast     BOOLEAN          NOT NULL DEFAULT FALSE,
    source          TEXT             NOT NULL DEFAULT 'OPEN_METEO',
    created_at      TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    UNIQUE (ts, location, role)
);

SELECT create_hypertable(
    'weather_hourly', 'ts',
    chunk_time_interval => INTERVAL '1 week',
    if_not_exists => TRUE
);

-- ─────────────────────────────────────────────────────────────
-- Table 4: Hourly KSE generation by source type
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS generation_hourly (
    ts              TIMESTAMPTZ      NOT NULL,
    source_type     TEXT             NOT NULL,
    value_mw        DOUBLE PRECISION NOT NULL,
    is_forecast     BOOLEAN          NOT NULL DEFAULT FALSE,
    data_source     TEXT             NOT NULL,
    created_at      TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    UNIQUE (ts, source_type)
);

SELECT create_hypertable(
    'generation_hourly', 'ts',
    chunk_time_interval => INTERVAL '1 week',
    if_not_exists => TRUE
);

-- ─────────────────────────────────────────────────────────────
-- Table 5: 15-minute curtailment data (PSE POZE-REDOZE)
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS curtailment_15min (
    ts              TIMESTAMPTZ      NOT NULL,
    wi_balance_mw   DOUBLE PRECISION NOT NULL DEFAULT 0,
    wi_network_mw   DOUBLE PRECISION NOT NULL DEFAULT 0,
    pv_balance_mw   DOUBLE PRECISION NOT NULL DEFAULT 0,
    pv_network_mw   DOUBLE PRECISION NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    UNIQUE (ts)
);

SELECT create_hypertable(
    'curtailment_15min', 'ts',
    chunk_time_interval => INTERVAL '1 month',
    if_not_exists => TRUE
);

-- ─────────────────────────────────────────────────────────────
-- Table 6: Reserve capacity prices (PSE cmbp-tp)
-- ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS reserve_prices_hourly (
    ts               TIMESTAMPTZ      NOT NULL,
    afrr_d_pln_mw   DOUBLE PRECISION,
    afrr_g_pln_mw   DOUBLE PRECISION,
    mfrrd_d_pln_mw  DOUBLE PRECISION,
    mfrrd_g_pln_mw  DOUBLE PRECISION,
    fcr_d_pln_mw    DOUBLE PRECISION,
    fcr_g_pln_mw    DOUBLE PRECISION,
    rr_g_pln_mw     DOUBLE PRECISION,
    created_at       TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    UNIQUE (ts)
);

SELECT create_hypertable(
    'reserve_prices_hourly', 'ts',
    chunk_time_interval => INTERVAL '1 month',
    if_not_exists => TRUE
);
