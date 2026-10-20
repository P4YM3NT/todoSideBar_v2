-- Getrennte Zustellhistorie pro Kanal und Termin, auch nach einem Neustart.
CREATE TABLE notification_deliveries (
    task_id      TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    due_on       TEXT NOT NULL,
    channel      TEXT NOT NULL CHECK (channel IN ('popup', 'email')),
    delivered_at TEXT NOT NULL,
    PRIMARY KEY (task_id, due_on, channel)
);
