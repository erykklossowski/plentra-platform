-- Monthly price aggregates (auto-refreshed, used by /api/summary history)
CREATE MATERIALIZED VIEW IF NOT EXISTS price_monthly
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 month', ts)  AS month,
    source,
    product,
    AVG(value_eur_mwh)          AS avg_eur,
    MIN(value_eur_mwh)          AS min_eur,
    MAX(value_eur_mwh)          AS max_eur,
    AVG(value_pln_mwh)          AS avg_pln,
    COUNT(*)                    AS sample_count
FROM price_hourly
WHERE is_forecast = FALSE
  AND value_eur_mwh IS NOT NULL
GROUP BY month, source, product
WITH NO DATA;

SELECT add_continuous_aggregate_policy('price_monthly',
    start_offset    => INTERVAL '3 months',
    end_offset      => INTERVAL '1 hour',
    schedule_interval => INTERVAL '1 day',
    if_not_exists   => TRUE
);

-- Daily curtailment aggregates
CREATE MATERIALIZED VIEW IF NOT EXISTS curtailment_daily
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 day', ts)    AS day,
    SUM(wi_balance_mw) * 0.25   AS wi_balance_mwh,
    SUM(wi_network_mw) * 0.25   AS wi_network_mwh,
    SUM(pv_balance_mw) * 0.25   AS pv_balance_mwh,
    SUM(pv_network_mw) * 0.25   AS pv_network_mwh,
    SUM((wi_balance_mw + wi_network_mw + pv_balance_mw + pv_network_mw)) * 0.25
                                AS total_mwh
FROM curtailment_15min
GROUP BY day
WITH NO DATA;

SELECT add_continuous_aggregate_policy('curtailment_daily',
    start_offset    => INTERVAL '2 months',
    end_offset      => INTERVAL '15 minutes',
    schedule_interval => INTERVAL '1 hour',
    if_not_exists   => TRUE
);

-- Monthly reserve price averages
CREATE MATERIALIZED VIEW IF NOT EXISTS reserve_prices_monthly
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 month', ts)  AS month,
    AVG(afrr_d_pln_mw)          AS afrr_d,
    AVG(afrr_g_pln_mw)          AS afrr_g,
    AVG(mfrrd_d_pln_mw)         AS mfrrd_d,
    AVG(mfrrd_g_pln_mw)         AS mfrrd_g,
    AVG(fcr_d_pln_mw)           AS fcr_d,
    AVG(fcr_g_pln_mw)           AS fcr_g,
    AVG(rr_g_pln_mw)            AS rr_g
FROM reserve_prices_hourly
GROUP BY month
WITH NO DATA;

SELECT add_continuous_aggregate_policy('reserve_prices_monthly',
    start_offset    => INTERVAL '14 months',
    end_offset      => INTERVAL '1 hour',
    schedule_interval => INTERVAL '1 day',
    if_not_exists   => TRUE
);
