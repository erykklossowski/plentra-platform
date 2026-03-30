-- Add PSE price columns to price_hourly
-- Safe to run multiple times (IF NOT EXISTS)
ALTER TABLE price_hourly
    ADD COLUMN IF NOT EXISTS cen_pln     DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS ckoeb_pln   DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS csdac_pln   DOUBLE PRECISION;

COMMENT ON COLUMN price_hourly.cen_pln
    IS 'CEN — rozliczeniowa cena energii (settlement price) PLN/MWh';
COMMENT ON COLUMN price_hourly.ckoeb_pln
    IS 'CKOEB — cena ko energii bilansującej (balancing market) PLN/MWh';
COMMENT ON COLUMN price_hourly.csdac_pln
    IS 'SDAC — cena dnia następnego z couplingu (DA coupling) PLN/MWh';
