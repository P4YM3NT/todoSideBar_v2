-- Links survive soft-deleted tasks so calendar copies can be removed later.
CREATE TABLE calendar_links (
    task_id TEXT NOT NULL,
    calendar_id TEXT NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (task_id, calendar_id)
);
