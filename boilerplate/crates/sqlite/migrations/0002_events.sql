-- Migration 0002: domain-event store
--
-- Persists immutable domain events for event-sourced aggregates.
-- `aggregate_id` links events back to their owning aggregate.
-- `event_type`   is a discriminant string (e.g. "OrderPlaced").
-- `payload`      is the full event as a JSON document.
-- `occurred_at`  is the wall-clock ISO-8601 timestamp at which the event
--                was raised.

CREATE TABLE IF NOT EXISTS events (
    id            TEXT    PRIMARY KEY NOT NULL,
    aggregate_id  TEXT    NOT NULL,
    event_type    TEXT    NOT NULL,
    payload       JSON    NOT NULL,
    occurred_at   TEXT    NOT NULL
);

-- Fast look-up of all events for a single aggregate (replaying its history).
CREATE INDEX IF NOT EXISTS idx_events_aggregate_id ON events (aggregate_id);

-- Chronological ordering across all events.
CREATE INDEX IF NOT EXISTS idx_events_occurred_at  ON events (occurred_at);

-- Filter by event type (e.g. projection catch-up).
CREATE INDEX IF NOT EXISTS idx_events_event_type   ON events (event_type);
