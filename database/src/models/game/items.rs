use common::time::ServerTime;
use sonettobuf;
use sqlx::{FromRow, SqlitePool};

#[allow(async_fn_in_trait)]
pub trait ItemModel<T> {
    async fn get_all(&self) -> Result<Vec<T>, sqlx::Error>;
    async fn get(&self, item_id: i64) -> Result<Option<T>, sqlx::Error>;
    async fn create(&self, item_id: i32, amount: i32) -> Result<Vec<i32>, sqlx::Error>;
    async fn update_quantity(&self, item_id: i32, delta: i32) -> Result<bool, sqlx::Error>;
}

pub struct UserItemModel {
    user_id: i64,
    pool: SqlitePool,
}

#[derive(Debug, Clone, FromRow)]
pub struct Item {
    pub user_id: i64,
    pub item_id: i64,
    pub quantity: i32,
    pub last_use_time: Option<i64>,
    pub last_update_time: Option<i64>,
    pub total_gain_count: i64,
}

impl From<Item> for sonettobuf::Item {
    fn from(item: Item) -> Self {
        sonettobuf::Item {
            item_id: Some(item.item_id as u32),
            quantity: Some(item.quantity),
            last_use_time: item.last_use_time.map(|t| t as u64),
            last_update_time: item.last_update_time.map(|t| t as u64),
            total_gain_count: Some(item.total_gain_count),
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct PowerItem {
    pub uid: i64,
    pub user_id: i64,
    pub item_id: i64,
    pub quantity: i32,
    pub expire_time: i32,
    pub created_at: i64,
}

impl From<PowerItem> for sonettobuf::PowerItem {
    fn from(item: PowerItem) -> Self {
        sonettobuf::PowerItem {
            uid: Some(item.uid),
            item_id: Some(item.item_id),
            quantity: Some(item.quantity),
            expire_time: Some(item.expire_time),
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct InsightItem {
    pub uid: i64,
    pub user_id: i64,
    pub item_id: i32,
    pub quantity: i32,
    pub expire_time: i32,
}

impl From<InsightItem> for sonettobuf::InsightItem {
    fn from(item: InsightItem) -> Self {
        sonettobuf::InsightItem {
            uid: Some(item.uid),
            item_id: Some(item.item_id),
            quantity: Some(item.quantity),
            expire_time: Some(item.expire_time),
        }
    }
}

impl UserItemModel {
    pub fn new(user_id: i64, pool: SqlitePool) -> Self {
        Self { user_id, pool }
    }
}

impl ItemModel<Item> for UserItemModel {
    async fn get_all(&self) -> Result<Vec<Item>, sqlx::Error> {
        sqlx::query_as::<_, Item>(
            "SELECT user_id, item_id, quantity, last_use_time, last_update_time, total_gain_count
             FROM items WHERE user_id = ? ORDER BY item_id",
        )
        .bind(self.user_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn get(&self, item_id: i64) -> Result<Option<Item>, sqlx::Error> {
        sqlx::query_as::<_, Item>(
            "SELECT user_id, item_id, quantity, last_use_time, last_update_time, total_gain_count
             FROM items WHERE user_id = ? AND item_id = ?",
        )
        .bind(self.user_id)
        .bind(item_id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn create(&self, item_id: i32, amount: i32) -> Result<Vec<i32>, sqlx::Error> {
        sqlx::query(
            "INSERT INTO items (user_id, item_id, quantity, last_update_time, total_gain_count)
             VALUES (?, ?, ?, ?, ?)
             ON CONFLICT(user_id, item_id) DO UPDATE SET
                 quantity = quantity + excluded.quantity,
                 last_update_time = excluded.last_update_time,
                 total_gain_count = total_gain_count + excluded.total_gain_count",
        )
        .bind(self.user_id)
        .bind(item_id)
        .bind(amount)
        .bind(ServerTime::now_ms())
        .bind(amount)
        .execute(&self.pool)
        .await?;

        Ok(vec![item_id])
    }

    async fn update_quantity(&self, item_id: i32, delta: i32) -> Result<bool, sqlx::Error> {
        let current: Option<i32> =
            sqlx::query_scalar("SELECT quantity FROM items WHERE user_id = ? AND item_id = ?")
                .bind(self.user_id)
                .bind(item_id as i64)
                .fetch_optional(&self.pool)
                .await?;

        let current_qty = current.unwrap_or(0);

        if delta < 0 && current_qty < delta.abs() {
            return Ok(false);
        }

        let timestamp = ServerTime::now_ms();
        sqlx::query(
            "UPDATE items
                 SET quantity = quantity + ?,
                     last_use_time = ?,
                     last_update_time = ?
                 WHERE user_id = ? AND item_id = ?",
        )
        .bind(delta)
        .bind(timestamp as i64)
        .bind(timestamp as i64)
        .bind(self.user_id)
        .bind(item_id as i64)
        .execute(&self.pool)
        .await?;

        Ok(true)
    }
}

impl ItemModel<PowerItem> for UserItemModel {
    async fn get_all(&self) -> Result<Vec<PowerItem>, sqlx::Error> {
        sqlx::query_as::<_, PowerItem>(
            "SELECT
                uid,
                user_id,
                item_id,
                quantity,
                expire_time,
                created_at
            FROM power_items
            WHERE user_id = ?
              AND expire_time = 0
            GROUP BY user_id, item_id

            UNION ALL


            SELECT
                MIN(uid)              AS uid,
                user_id               AS user_id,
                item_id               AS item_id,
                SUM(quantity)         AS quantity,
                MIN(expire_time)      AS expire_time,
                MIN(created_at)       AS created_at
            FROM power_items
            WHERE user_id = ?
              AND expire_time > CAST(strftime('%s','now') AS INTEGER)
            GROUP BY user_id, item_id

            ORDER BY expire_time ASC",
        )
        .bind(self.user_id)
        .bind(self.user_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn get(&self, item_id: i64) -> Result<Option<PowerItem>, sqlx::Error> {
        sqlx::query_as::<_, PowerItem>(
            "SELECT uid, user_id, item_id, quantity, expire_time, created_at
             FROM power_items WHERE user_id = ? AND item_id = ?",
        )
        .bind(self.user_id)
        .bind(item_id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn create(&self, item_id: i32, amount: i32) -> Result<Vec<i32>, sqlx::Error> {
        let mut changed_item_ids = Vec::new();
        let now = common::time::ServerTime::now_ms();
        let game_data = data::exceldb::get();

        let power_item_config = game_data.power_item.iter().find(|p| p.id == item_id);
        let expire_time = if let Some(config) = power_item_config {
            match config.expire_type {
                0 => 0,
                1 | 2 | 3 => (now / 1000) + (10 * 24 * 60 * 60),
                _ => 0,
            }
        } else {
            0
        };

        for _ in 0..amount {
            sqlx::query(
                "INSERT INTO power_items (user_id, item_id, quantity, expire_time, created_at)
                     VALUES (?, ?, 1, ?, ?)",
            )
            .bind(self.user_id)
            .bind(item_id)
            .bind(expire_time)
            .bind(now)
            .execute(&self.pool)
            .await?;
        }

        changed_item_ids.push(item_id);
        tracing::info!(
            "Added {} power items (id: {}) to user {} with expire_time: {}",
            amount,
            item_id,
            self.user_id,
            expire_time
        );

        Ok(changed_item_ids)
    }

    async fn update_quantity(&self, item_id: i32, delta: i32) -> Result<bool, sqlx::Error> {
        let current: Option<i64> = sqlx::query_scalar(
            "SELECT SUM(quantity) FROM power_items WHERE user_id = ? AND item_id = ?",
        )
        .bind(self.user_id)
        .bind(item_id)
        .fetch_optional(&self.pool)
        .await?;

        let current_qty = current.unwrap_or(0) as i32;

        if delta < 0 && current_qty < delta.abs() {
            return Ok(false);
        }

        if delta < 0 {
            let to_remove = delta.abs();
            sqlx::query(
                "DELETE FROM power_items
                     WHERE uid IN (
                         SELECT uid FROM power_items
                         WHERE user_id = ? AND item_id = ?
                         ORDER BY created_at ASC
                         LIMIT ?
                     )",
            )
            .bind(self.user_id)
            .bind(item_id)
            .bind(to_remove)
            .execute(&self.pool)
            .await?;
        } else if delta > 0 {
            ItemModel::<PowerItem>::create(self, item_id, delta).await?;
        }

        Ok(true)
    }
}

impl ItemModel<InsightItem> for UserItemModel {
    async fn get_all(&self) -> Result<Vec<InsightItem>, sqlx::Error> {
        sqlx::query_as::<_, InsightItem>(
            "SELECT uid, user_id, item_id, quantity, expire_time
             FROM insight_items WHERE user_id = ?
             ORDER BY expire_time",
        )
        .bind(self.user_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn get(&self, item_id: i64) -> Result<Option<InsightItem>, sqlx::Error> {
        sqlx::query_as::<_, InsightItem>(
            "SELECT uid, user_id, item_id, quantity, expire_time
             FROM insight_items WHERE user_id = ? AND item_id = ?",
        )
        .bind(self.user_id)
        .bind(item_id as i32)
        .fetch_optional(&self.pool)
        .await
    }

    async fn create(&self, item_id: i32, amount: i32) -> Result<Vec<i32>, sqlx::Error> {
        let now = common::time::ServerTime::now_ms();
        let game_data = data::exceldb::get();

        let insight_config = game_data.insight_item.iter().find(|i| i.id == item_id);
        let expire_time = if let Some(config) = insight_config {
            let hours = config.expire_hours as i64;
            ((now / 1000) + (hours * 60 * 60)) as i32
        } else {
            0
        };

        for _ in 0..amount {
            sqlx::query(
                "INSERT INTO insight_items (user_id, item_id, quantity, expire_time)
                     VALUES (?, ?, 1, ?)",
            )
            .bind(self.user_id)
            .bind(item_id)
            .bind(expire_time)
            .execute(&self.pool)
            .await?;
        }

        tracing::info!(
            "Added {} insight items (id: {}) to user {} with expire_time: {}",
            amount,
            item_id,
            self.user_id,
            expire_time
        );

        Ok(vec![item_id])
    }

    async fn update_quantity(&self, item_id: i32, delta: i32) -> Result<bool, sqlx::Error> {
        let current: Option<i64> = sqlx::query_scalar(
            "SELECT SUM(quantity) FROM insight_items WHERE user_id = ? AND item_id = ?",
        )
        .bind(self.user_id)
        .bind(item_id)
        .fetch_optional(&self.pool)
        .await?;

        let current_qty = current.unwrap_or(0) as i32;

        if delta < 0 && current_qty < delta.abs() {
            return Ok(false);
        }

        if delta < 0 {
            let to_remove = delta.abs();
            sqlx::query(
                "DELETE FROM insight_items
                     WHERE uid IN (
                         SELECT uid FROM insight_items
                         WHERE user_id = ? AND item_id = ?
                         ORDER BY uid ASC
                         LIMIT ?
                     )",
            )
            .bind(self.user_id)
            .bind(item_id)
            .bind(to_remove)
            .execute(&self.pool)
            .await?;
        } else if delta > 0 {
            ItemModel::<InsightItem>::create(self, item_id, delta).await?;
        }

        Ok(true)
    }
}

impl UserItemModel {
    pub async fn get_item(&self, item_id: u32) -> Result<Option<Item>, sqlx::Error> {
        ItemModel::<Item>::get(self, item_id as i64).await
    }
    
    pub async fn get_all_items(&self) -> Result<Vec<Item>, sqlx::Error> {
        ItemModel::<Item>::get_all(self).await
    }

    pub async fn get_all_power_items(&self) -> Result<Vec<PowerItem>, sqlx::Error> {
        ItemModel::<PowerItem>::get_all(self).await
    }

    pub async fn get_all_insight_items(&self) -> Result<Vec<InsightItem>, sqlx::Error> {
        ItemModel::<InsightItem>::get_all(self).await
    }

    pub async fn create_items(&self, items: &[(u32, i32)]) -> Result<Vec<i32>, sqlx::Error> {
        let mut all_changed_ids = Vec::new();
        for (item_id, amount) in items {
            let changed = ItemModel::<Item>::create(self, *item_id as i32, *amount).await?;
            all_changed_ids.extend(changed);
        }
        Ok(all_changed_ids)
    }

    pub async fn create_power_items(&self, items: &[(i32, i32)]) -> Result<Vec<i32>, sqlx::Error> {
        let mut all_changed_ids = Vec::new();
        for (item_id, amount) in items {
            let changed = ItemModel::<PowerItem>::create(self, *item_id, *amount).await?;
            all_changed_ids.extend(changed);
        }
        Ok(all_changed_ids)
    }
    
    pub async fn create_insight_items(&self, items: &[(i32, i32)]) -> Result<Vec<i32>, sqlx::Error> {
        let mut all_changed_ids = Vec::new();
        for (item_id, amount) in items {
            let changed = ItemModel::<InsightItem>::create(self, *item_id, *amount).await?;
            all_changed_ids.extend(changed);
        }
        Ok(all_changed_ids)
    }

    pub async fn update_item_quantity(
        &self,
        item_id: i32,
        delta: i32,
    ) -> Result<bool, sqlx::Error> {
        ItemModel::<Item>::update_quantity(self, item_id, delta).await
    }

    pub async fn add_item_quantity(&self, item_id: i32, amount: i32) -> Result<bool, sqlx::Error> {
        self.update_item_quantity(item_id, amount).await
    }

    pub async fn remove_item_quantity(
        &self,
        item_id: u32,
        amount: i32,
    ) -> Result<bool, sqlx::Error> {
        self.update_item_quantity(item_id as i32, -amount).await
    }

    pub async fn update_power_item_quantity(
        &self,
        item_id: i32,
        delta: i32,
    ) -> Result<bool, sqlx::Error> {
        ItemModel::<PowerItem>::update_quantity(self, item_id, delta).await
    }

    pub async fn add_power_item_quantity(
        &self,
        item_id: i32,
        amount: i32,
    ) -> Result<bool, sqlx::Error> {
        self.update_power_item_quantity(item_id, amount).await
    }

    pub async fn remove_power_item_quantity(
        &self,
        item_id: i32,
        amount: i32,
    ) -> Result<bool, sqlx::Error> {
        self.update_power_item_quantity(item_id, -amount).await
    }

    pub async fn update_insight_item_quantity(
        &self,
        item_id: i32,
        delta: i32,
    ) -> Result<bool, sqlx::Error> {
        ItemModel::<InsightItem>::update_quantity(self, item_id, delta).await
    }

    pub async fn add_insight_item_quantity(
        &self,
        item_id: i32,
        amount: i32,
    ) -> Result<bool, sqlx::Error> {
        self.update_insight_item_quantity(item_id, amount).await
    }

    pub async fn remove_insight_item_quantity(
        &self,
        item_id: i32,
        amount: i32,
    ) -> Result<bool, sqlx::Error> {
        self.update_insight_item_quantity(item_id, -amount).await
    }
}
