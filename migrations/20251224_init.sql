-- Create profiles table
CREATE TABLE IF NOT EXISTS profiles (
    client_id TEXT PRIMARY KEY,
    fiscal_category TEXT NOT NULL, -- e.g., 'RESPONSABLE_INSCRIPTO', 'MONOTRIBUTO'
    config JSONB NOT NULL DEFAULT '{}'
);

-- Create IVA rates by jurisdiction
CREATE TABLE IF NOT EXISTS iva_rates (
    jurisdiction TEXT PRIMARY KEY,
    rate FLOAT8 NOT NULL
);

-- Seed IVA rates
INSERT INTO iva_rates (jurisdiction, rate)
VALUES 
    ('TIERRA_DEL_FUEGO', 0.105),
    ('BUENOS_AIRES', 0.21),
    ('DEFAULT', 0.21)
ON CONFLICT (jurisdiction) DO NOTHING;

-- Basic indexes as per RT-002
CREATE INDEX IF NOT EXISTS idx_profiles_fiscal_category ON profiles(fiscal_category);

-- Insert sample data for POC
INSERT INTO profiles (client_id, fiscal_category, config)
VALUES
    ('client_1', 'RESPONSABLE_INSCRIPTO', '{"name": "Empresa A"}'),
    ('client_2', 'MONOTRIBUTO', '{"name": "Juan Perez"}')
ON CONFLICT (client_id) DO NOTHING;
