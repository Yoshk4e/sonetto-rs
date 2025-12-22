CREATE TABLE IF NOT EXISTS user_gacha_state (
    user_id         INTEGER NOT NULL,
    pool_id         INTEGER NOT NULL,

    pity_6          INTEGER NOT NULL DEFAULT 0,  -- pulls since last 6*
    up_guaranteed   INTEGER NOT NULL DEFAULT 0,  -- 0 = false, 1 = true

    last_pull_at    INTEGER,                     -- unix sec (optional, analytics/debug)

    PRIMARY KEY (user_id, pool_id),

    FOREIGN KEY (user_id)
        REFERENCES users(id)
        ON DELETE CASCADE
);
