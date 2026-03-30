-- Raw daily OHLCV bars fetched from Databento ohlcv-1d schema.
-- One row per (date, instrument_id). No fallbacks, no NULLs for prices —
-- a missing row means no trading occurred that day.
CREATE TABLE IF NOT EXISTS fuel_ohlcv (
    date          DATE             NOT NULL,
    instrument_id BIGINT           NOT NULL,
    dataset       TEXT             NOT NULL,  -- 'IFEU.IMPACT'
    ticker        TEXT             NOT NULL,  -- 'TTF', 'EUA', 'ARA', 'GAB'
    raw_symbol    TEXT             NOT NULL,  -- e.g. 'TFM FMJ0026!'
    unit          TEXT             NOT NULL,
    open          DOUBLE PRECISION NOT NULL,
    high          DOUBLE PRECISION NOT NULL,
    low           DOUBLE PRECISION NOT NULL,
    close         DOUBLE PRECISION NOT NULL,  -- CSS uses this field
    volume        BIGINT           NOT NULL,
    PRIMARY KEY (date, instrument_id, dataset)
);

SELECT create_hypertable('fuel_ohlcv', 'date',
    chunk_time_interval => INTERVAL '3 months',
    if_not_exists => TRUE);

-- Fast lookup: all contracts for one ticker by date
CREATE INDEX IF NOT EXISTS fuel_ohlcv_ticker_date ON fuel_ohlcv (ticker, date DESC);

-- Fast lookup: one contract's time series
CREATE INDEX IF NOT EXISTS fuel_ohlcv_raw_symbol_date ON fuel_ohlcv (raw_symbol, date DESC);

COMMENT ON TABLE fuel_ohlcv IS
    'Daily OHLCV bars from Databento ohlcv-1d schema. '
    'One row per (date, instrument_id). Missing row = no trading that day. '
    'CSS calculation uses the close price field.';

-- Calculated spreads (CSS results)
CREATE TABLE IF NOT EXISTS calculated_spreads (
    date          DATE             NOT NULL,
    spread_type   TEXT             NOT NULL,  -- 'rolling_3m_css'
    value         DOUBLE PRECISION NOT NULL,

    -- Components for auditability
    power_avg     DOUBLE PRECISION NOT NULL,
    gas_avg       DOUBLE PRECISION NOT NULL,
    carbon_price  DOUBLE PRECISION NOT NULL,

    -- Exact symbols used in this calculation
    power_symbols TEXT[]           NOT NULL,  -- 3 GAB symbols
    gas_symbols   TEXT[]           NOT NULL,  -- 3 TFM symbols
    carbon_symbol TEXT             NOT NULL,  -- 1 ECF symbol

    calculated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (date, spread_type)
);

COMMENT ON TABLE calculated_spreads IS
    'Daily CSS values. '
    'CSS = Power_avg - Gas_avg/0.50 - Carbon*0.202/0.50. '
    'Calculation fails and nothing is written if any of the 7 prices is missing.';
