CREATE TABLE IF NOT EXISTS user_summon_history (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,

    user_id         INTEGER NOT NULL,
    pool_id         INTEGER NOT NULL,

    summon_type     INTEGER NOT NULL,      -- 1 or 10
    pool_type       INTEGER NOT NULL,      -- 3
    pool_name       TEXT NOT NULL,

    summon_time     INTEGER NOT NULL,      -- unix timestamp

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, pool_id, summon_time)
);

CREATE INDEX idx_user_summon_history_user
    ON user_summon_history(user_id);

CREATE INDEX idx_user_summon_history_pool
    ON user_summon_history(pool_id);

CREATE TABLE IF NOT EXISTS user_summon_history_items (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,

    history_id      INTEGER NOT NULL,
    result_index    INTEGER NOT NULL,   -- preserves order (0..9)
    gain_id         INTEGER NOT NULL,

    FOREIGN KEY (history_id) REFERENCES user_summon_history(id) ON DELETE CASCADE,

    UNIQUE(history_id, result_index)
);

CREATE INDEX idx_summon_items_history
    ON user_summon_history_items(history_id);

CREATE INDEX idx_summon_items_gain
    ON user_summon_history_items(gain_id);
