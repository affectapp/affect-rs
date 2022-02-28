use crate::{Error, PgPool};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use std::sync::Arc;
use uuid::Uuid;

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

    async fn find_user_by_firebase_uid(
        &self,
        firebase_uid: String,
    ) -> Result<Option<UserRow>, Error>;
}

pub struct PgUserStore {
    pool: Arc<PgPool>,
}

impl PgUserStore {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserStore for PgUserStore {
    async fn add_user(&self, new_user: NewUserRow) -> Result<UserRow, Error> {
        Ok(sqlx::query_file_as!(
            UserRow,
            "queries/user/insert.sql",
            &new_user.create_time,
            &new_user.update_time,
            &new_user.firebase_uid,
            &new_user.firebase_email,
        )
        .fetch_one(self.pool.inner())
        .await?)
    }

    async fn find_user_by_firebase_uid(
        &self,
        firebase_uid: String,
    ) -> Result<Option<UserRow>, Error> {
        Ok(sqlx::query_file_as!(
            UserRow,
            "queries/user/find_by_firebase_uid.sql",
            &firebase_uid,
        )
        .fetch_optional(self.pool.inner())
        .await?)
    }
}
