use common::time::ServerTime;
use sonettobuf;
use sqlx::{FromRow, SqlitePool};

#[allow(async_fn_in_trait)]
pub trait CurrencyModel<T> {
    async fn get_all(&self) -> Result<Vec<T>, sqlx::Error>;
    async fn get(&self, currency_id: i32) -> Result<Option<T>, sqlx::Error>;
    async fn create(&self, currency_id: i32, amount: i32) -> Result<Vec<i32>, sqlx::Error>;
    async fn update_quantity(&self, currency_id: i32, delta: i32) -> Result<bool, sqlx::Error>;
}

pub struct UserCurrencyModel {
    user_id: i64,
    pool: SqlitePool,
}

#[derive(Debug, Clone, FromRow)]
pub struct Currency {
    pub user_id: i64,
    pub currency_id: i32,
    pub quantity: i32,
    pub last_recover_time: Option<i64>,
    pub expired_time: Option<i64>,
}

impl From<Currency> for sonettobuf::Currency {
    fn from(c: Currency) -> Self {
        sonettobuf::Currency {
            currency_id: Some(c.currency_id as u32),
            quantity: Some(c.quantity),
            last_recover_time: c.last_recover_time.map(|t| t as u64),
            expired_time: c.expired_time.map(|t| t as u64),
        }
    }
}

impl UserCurrencyModel {
    pub fn new(user_id: i64, pool: SqlitePool) -> Self {
        Self { user_id, pool }
    }
}

impl CurrencyModel<Currency> for UserCurrencyModel {
    async fn get_all(&self) -> Result<Vec<Currency>, sqlx::Error> {
        sqlx::query_as::<_, Currency>(
            "SELECT user_id, currency_id, quantity, last_recover_time, expired_time
             FROM currencies
             WHERE user_id = ? ORDER BY ORDER BY currency_id",
        )
        .bind(self.user_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn get(&self, currency_id: i32) -> Result<Option<Currency>, sqlx::Error> {
        sqlx::query_as::<_, Currency>(
            "SELECT user_id, currency_id, quantity, last_recover_time, expired_time
             FROM currencies
             WHERE user_id = ? AND currency_id = ?",
        )
        .bind(self.user_id)
        .bind(currency_id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn create(&self, currency_id: i32, amount: i32) -> Result<Vec<i32>, sqlx::Error> {
        sqlx::query(
            "INSERT INTO currencies (user_id, currency_id, quantity, last_recover_time, expired_time)
             VALUES (?, ?, ?, ?, 0)
             ON CONFLICT(user_id, currency_id) DO UPDATE SET
                 quantity = quantity + excluded.quantity,
                 last_recover_time = excluded.last_recover_time",
        )
        .bind(self.user_id)
        .bind(currency_id)
        .bind(amount)
        .bind(ServerTime::now_ms())
        .bind(amount)
        .execute(&self.pool)
        .await?;

        Ok(vec![currency_id])
    }

    async fn update_quantity(&self, currency_id: i32, delta: i32) -> Result<bool, sqlx::Error> {
        let current: Option<i32> = sqlx::query_scalar(
            "SELECT quantity FROM currencies WHERE user_id = ? AND currency_id = ?",
        )
        .bind(self.user_id)
        .bind(currency_id)
        .fetch_optional(&self.pool)
        .await?;

        let current_qty = current.unwrap_or(0);

        if delta < 0 && current_qty < delta.abs() {
            return Ok(false);
        }

        let timestamp = ServerTime::now_ms();
        sqlx::query(
            "UPDATE currencies
                 SET quantity = quantity + ?,
                     last_recover_time = ?
                 WHERE user_id = ? AND currency_id = ?",
        )
        .bind(delta)
        .bind(timestamp)
        .bind(self.user_id)
        .bind(currency_id)
        .execute(&self.pool)
        .await?;

        Ok(true)
    }
}

impl UserCurrencyModel {
    pub async fn get_currency(&self, currency_id: i32) -> Result<Option<Currency>, sqlx::Error> {
        CurrencyModel::<Currency>::get(self, currency_id).await
    }

    pub async fn get_all_currencies(&self) -> Result<Vec<Currency>, sqlx::Error> {
        CurrencyModel::<Currency>::get_all(self).await
    }

    pub async fn update_currency(&self, currency_id: i32, delta: i32) -> Result<bool, sqlx::Error> {
        CurrencyModel::<Currency>::update_quantity(self, currency_id, delta).await
    }

    pub async fn add_currency(&self, currency_id: i32, amount: i32) -> Result<bool, sqlx::Error> {
        self.update_currency(currency_id, amount).await
    }

    pub async fn remove_currency(
        &self,
        currency_id: i32,
        amount: i32,
    ) -> Result<bool, sqlx::Error> {
        self.update_currency(currency_id, -amount).await
    }

    pub async fn create_currencies(
        &self,
        currencies: &[(i32, i32)],
    ) -> Result<Vec<(i32, i32)>, sqlx::Error> {
        let mut changes = Vec::new();
        for (currency_id, amount) in currencies {
            CurrencyModel::<Currency>::create(self, *currency_id, *amount).await?;
            changes.push((*currency_id, *amount));
        }
        Ok(changes)
    }
}
