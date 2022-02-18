use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{postgres::PgPoolOptions, FromRow, Pool, Postgres};
use uuid::Uuid;

use crate::Error;

#[derive(Clone, Debug, FromRow)]
pub struct UserRow {
    pub user_id: Uuid,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub firebase_uid: String,
    pub firebase_email: String,
}

pub struct NewUserRow {
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub firebase_uid: String,
    pub firebase_email: String,
}

#[async_trait]
pub trait UserStore: Sync + Send {
    async fn add_user(&self, new_user: NewUserRow) -> Result<UserRow, Error>;

    async fn get_user_by_firebase_uid(
        &self,
        firebase_uid: String,
    ) -> Result<Option<UserRow>, Error>;
}

pub struct PostgresUserStore {
    pool: Arc<Pool<Postgres>>,
}

impl PostgresUserStore {
    pub fn new(pool: Arc<Pool<Postgres>>) -> Self {
        Self { pool }
    }

    pub async fn connect(postgres_uri: String) -> Result<Self, Error> {
        let pool = Arc::new(
            PgPoolOptions::new()
                .max_connections(2)
                .connect(&postgres_uri)
                .await?,
        );
        Ok(Self::new(pool))
    }
}

#[async_trait]
impl UserStore for PostgresUserStore {
    async fn add_user(&self, new_user: NewUserRow) -> Result<UserRow, Error> {
        Ok(sqlx::query_as(
            "INSERT INTO users (create_time, update_time, firebase_uid, firebase_email) \
            VALUES ($1, $2, $3, $4) \
            RETURNING *",
        )
        .bind(&new_user.create_time)
        .bind(&new_user.update_time)
        .bind(&new_user.firebase_uid)
        .bind(&new_user.firebase_email)
        .fetch_one(self.pool.as_ref())
        .await?)
    }

    async fn get_user_by_firebase_uid(
        &self,
        firebase_uid: String,
    ) -> Result<Option<UserRow>, Error> {
        Ok(sqlx::query_as(
            "SELECT * \
            FROM users \
            WHERE firebase_uid = $1",
        )
        .bind(firebase_uid)
        .fetch_optional(self.pool.as_ref())
        .await?)
    }
}
