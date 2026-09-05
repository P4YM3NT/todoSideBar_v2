-- Alle IDs sind UUIDv7 (zeitsortierbar, geräteübergreifend kollisionsfrei).
-- updated_at + deleted_at existieren ab Tag 1, damit späteres Sync keine Migration braucht.
-- sort_order ist REAL: Umsortieren = ein einziges UPDATE (Mittelwert der Nachbarn).

CREATE TABLE projects (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    color       TEXT,
    icon        TEXT,
    sort_order  REAL NOT NULL,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    deleted_at  TEXT
);

CREATE TABLE tasks (
    id           TEXT PRIMARY KEY,
    title        TEXT NOT NULL,
    notes        TEXT,
    project_id   TEXT REFERENCES projects(id) ON DELETE SET NULL,
    status       TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'done')),
    scheduled_on TEXT,          -- "wann arbeite ich dran" -> Planung
    due_on       TEXT,          -- harte Deadline
    priority     INTEGER NOT NULL DEFAULT 0,
    sort_order   REAL NOT NULL,
    completed_at TEXT,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL,
    deleted_at   TEXT
);

CREATE TABLE tags (
    id    TEXT PRIMARY KEY,
    name  TEXT NOT NULL UNIQUE,
    color TEXT
);

CREATE TABLE task_tags (
    task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    tag_id  TEXT NOT NULL REFERENCES tags(id)  ON DELETE CASCADE,
    PRIMARY KEY (task_id, tag_id)
);

CREATE TABLE settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL          -- JSON
);

-- Deckt die Hauptansichten ab: Inbox, Heute, Demnächst, Projekt.
CREATE INDEX idx_tasks_status_sched ON tasks (status, scheduled_on) WHERE deleted_at IS NULL;
CREATE INDEX idx_tasks_status_due   ON tasks (status, due_on)       WHERE deleted_at IS NULL;
CREATE INDEX idx_tasks_project      ON tasks (project_id, sort_order) WHERE deleted_at IS NULL;
