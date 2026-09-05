-- Kurze Beschreibung (einzeilig, erscheint in der Liste unter dem Titel).
-- `notes` bleibt das lange Freitextfeld aus 001.
ALTER TABLE tasks ADD COLUMN description TEXT;

-- Geschätzter Aufwand in Minuten.
ALTER TABLE tasks ADD COLUMN estimate_minutes INTEGER;

-- Checkliste innerhalb einer Aufgabe. Bewusst eine eigene Tabelle statt einer
-- Selbstreferenz auf `tasks`: Unterpunkte haben keine Termine, keine Projekte
-- und keine eigene Planung - als vollwertige Tasks wären fast alle Spalten leer.
CREATE TABLE subtasks (
    id         TEXT PRIMARY KEY,
    task_id    TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    title      TEXT NOT NULL,
    done       INTEGER NOT NULL DEFAULT 0,
    sort_order REAL NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX idx_subtasks_task ON subtasks (task_id, sort_order);

CREATE TABLE task_links (
    id         TEXT PRIMARY KEY,
    task_id    TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    url        TEXT NOT NULL,
    label      TEXT,
    sort_order REAL NOT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX idx_task_links_task ON task_links (task_id, sort_order);

-- Anhänge werden in den App-Ordner kopiert (Unterordner `attachments`).
-- `stored_name` ist der Name dort, `original_name` nur für die Anzeige - so
-- kollidieren zwei gleichnamige Dateien nicht, und das Original darf weg.
CREATE TABLE attachments (
    id            TEXT PRIMARY KEY,
    task_id       TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    original_name TEXT NOT NULL,
    stored_name   TEXT NOT NULL UNIQUE,
    mime_type     TEXT,
    size_bytes    INTEGER,
    created_at    TEXT NOT NULL
);
CREATE INDEX idx_attachments_task ON attachments (task_id);
