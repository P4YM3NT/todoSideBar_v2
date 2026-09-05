-- Unteraufgaben sind jetzt echte Aufgaben mit Elternbezug.
--
-- 002 hatte dafuer eine eigene Tabelle, weil Unterpunkte damals weder Termine
-- noch Projekte haben sollten. Sie sollen aber vollwertig sein - also faellt die
-- Sondertabelle weg. Sie wurde nie beschrieben, es gehen keine Daten verloren.
DROP TABLE subtasks;

ALTER TABLE tasks ADD COLUMN parent_id TEXT REFERENCES tasks(id);

CREATE INDEX idx_tasks_parent ON tasks (parent_id, sort_order) WHERE deleted_at IS NULL;

-- Pro Projekt einstellbar, ob Unteraufgaben in der Liste auftauchen.
-- 0 = nur Hauptaufgaben, wie in den festen Ansichten voreingestellt.
ALTER TABLE projects ADD COLUMN show_subtasks INTEGER NOT NULL DEFAULT 0;
