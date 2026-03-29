-- Compound indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_price_source_product
    ON price_hourly (source, product, ts DESC);

CREATE INDEX IF NOT EXISTS idx_fuel_ticker
    ON fuel_daily (ticker, ts DESC);

CREATE INDEX IF NOT EXISTS idx_weather_location
    ON weather_hourly (location, ts DESC);

CREATE INDEX IF NOT EXISTS idx_generation_source_type
    ON generation_hourly (source_type, ts DESC);

-- Enable compression on old chunks (>30 days)
ALTER TABLE price_hourly       SET (timescaledb.compress, timescaledb.compress_segmentby = 'source,product');
ALTER TABLE fuel_daily         SET (timescaledb.compress, timescaledb.compress_segmentby = 'ticker');
ALTER TABLE weather_hourly     SET (timescaledb.compress, timescaledb.compress_segmentby = 'location');
ALTER TABLE generation_hourly  SET (timescaledb.compress, timescaledb.compress_segmentby = 'source_type');
ALTER TABLE curtailment_15min  SET (timescaledb.compress);
ALTER TABLE reserve_prices_hourly SET (timescaledb.compress);

SELECT add_compression_policy('price_hourly',      INTERVAL '30 days', if_not_exists => TRUE);
SELECT add_compression_policy('fuel_daily',         INTERVAL '90 days', if_not_exists => TRUE);
SELECT add_compression_policy('weather_hourly',     INTERVAL '30 days', if_not_exists => TRUE);
SELECT add_compression_policy('generation_hourly',  INTERVAL '30 days', if_not_exists => TRUE);
SELECT add_compression_policy('curtailment_15min',  INTERVAL '90 days', if_not_exists => TRUE);
SELECT add_compression_policy('reserve_prices_hourly', INTERVAL '90 days', if_not_exists => TRUE);
