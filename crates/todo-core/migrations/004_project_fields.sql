-- `icon` gibt es schon aus 001; hier kommt der Rest dazu.
ALTER TABLE projects ADD COLUMN description TEXT;
ALTER TABLE projects ADD COLUMN due_on TEXT;

-- Archivieren statt loeschen: das Projekt verschwindet aus der Liste, bleibt
-- aber samt Aufgaben erhalten und ist wieder hervorholbar. `deleted_at` bleibt
-- dem endgueltigen Loeschen vorbehalten.
ALTER TABLE projects ADD COLUMN archived_at TEXT;

CREATE INDEX idx_projects_active ON projects (sort_order)
    WHERE deleted_at IS NULL AND archived_at IS NULL;
