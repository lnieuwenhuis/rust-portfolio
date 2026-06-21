CREATE TABLE IF NOT EXISTS projects (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    slug TEXT UNIQUE NOT NULL,
    summary TEXT NOT NULL,
    body_markdown TEXT NOT NULL DEFAULT '',
    role TEXT,
    status TEXT NOT NULL DEFAULT 'In progress',
    tech_stack JSONB NOT NULL DEFAULT '[]'::jsonb,
    github_url TEXT,
    live_url TEXT,
    image_url TEXT,
    accent TEXT,
    published BOOLEAN NOT NULL DEFAULT false,
    featured BOOLEAN NOT NULL DEFAULT false,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_projects_public_order
    ON projects (published, featured DESC, display_order ASC, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_projects_display_order
    ON projects (display_order ASC);

CREATE OR REPLACE FUNCTION set_projects_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS projects_updated_at ON projects;

CREATE TRIGGER projects_updated_at
BEFORE UPDATE ON projects
FOR EACH ROW
EXECUTE FUNCTION set_projects_updated_at();
