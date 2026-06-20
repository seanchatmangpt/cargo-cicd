-- Migration 0001: generic JSON-blob entity store
--
-- Stores any domain aggregate as a JSON document.  `id` is the aggregate
-- identifier (UUID or any opaque string).  `data` holds the serialised value.
-- `created_at` and `updated_at` are ISO-8601 timestamps in UTC.

CREATE TABLE IF NOT EXISTS items (
    id          TEXT    PRIMARY KEY NOT NULL,
    data        JSON             NOT NULL,
    created_at  TEXT             NOT NULL,
    updated_at  TEXT             NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_items_created_at ON items (created_at);
