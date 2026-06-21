CREATE TABLE IF NOT EXISTS admin_credentials (
    username TEXT PRIMARY KEY,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE OR REPLACE FUNCTION set_admin_credentials_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS admin_credentials_updated_at ON admin_credentials;

CREATE TRIGGER admin_credentials_updated_at
BEFORE UPDATE ON admin_credentials
FOR EACH ROW
EXECUTE FUNCTION set_admin_credentials_updated_at();
