CREATE TABLE banner_schedule (
    pool_id        INTEGER PRIMARY KEY,
    online_time    INTEGER NOT NULL,
    offline_time   INTEGER NOT NULL,
    created_at     INTEGER NOT NULL,
    updated_at     INTEGER NOT NULL
);
