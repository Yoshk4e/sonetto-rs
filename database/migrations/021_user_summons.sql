-- Main summon stats
CREATE TABLE user_summon_stats (
    user_id INTEGER PRIMARY KEY,
    free_equip_summon BOOLEAN NOT NULL DEFAULT 0,
    is_show_new_summon BOOLEAN NOT NULL DEFAULT 0,
    new_summon_count INTEGER NOT NULL DEFAULT 0,
    total_summon_count INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Summon pool info (one per pool)
CREATE TABLE user_summon_pools (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    pool_id INTEGER NOT NULL,
    online_time INTEGER NOT NULL DEFAULT 0,
    offline_time INTEGER NOT NULL DEFAULT 0,
    have_free BOOLEAN NOT NULL DEFAULT 0,
    used_free_count INTEGER NOT NULL DEFAULT 0,
    discount_time INTEGER NOT NULL DEFAULT 0,
    can_get_guarantee_sr_count INTEGER NOT NULL DEFAULT 0,
    guarantee_sr_countdown INTEGER NOT NULL DEFAULT 0,
    summon_count INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, pool_id)
);

-- Lucky bag info per pool
CREATE TABLE user_lucky_bags (
    user_id INTEGER NOT NULL,
    pool_id INTEGER NOT NULL,
    count INTEGER NOT NULL DEFAULT 0,
    not_ssr_count INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, pool_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Individual bag status
CREATE TABLE user_single_bags (
    user_id INTEGER NOT NULL,
    pool_id INTEGER NOT NULL,
    bag_id INTEGER NOT NULL,
    is_open BOOLEAN NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, pool_id, bag_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Special pool info
CREATE TABLE user_sp_pool_info (
    user_id INTEGER NOT NULL,
    pool_id INTEGER NOT NULL,
    sp_type INTEGER NOT NULL DEFAULT 0,
    limited_ticket_id INTEGER NOT NULL DEFAULT 0,
    limited_ticket_num INTEGER NOT NULL DEFAULT 0,
    open_time INTEGER NOT NULL DEFAULT 0,
    used_first_ssr_guarantee BOOLEAN NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, pool_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Up hero IDs for special pools
CREATE TABLE user_sp_pool_up_heroes (
    user_id INTEGER NOT NULL,
    pool_id INTEGER NOT NULL,
    hero_id INTEGER NOT NULL,
    PRIMARY KEY (user_id, pool_id, hero_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Reward progress for special pools
CREATE TABLE user_sp_pool_reward_progress (
    user_id INTEGER NOT NULL,
    pool_id INTEGER NOT NULL,
    progress_id INTEGER NOT NULL,
    PRIMARY KEY (user_id, pool_id, progress_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_user_summon_pools ON user_summon_pools(user_id);
CREATE INDEX idx_user_lucky_bags ON user_lucky_bags(user_id);
CREATE INDEX idx_user_sp_pool_info ON user_sp_pool_info(user_id);
