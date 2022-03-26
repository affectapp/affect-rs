use crate::{sqlx::store::PgOnDemandStore, Error};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, FromRow)]
pub struct UserRow {
    pub user_id: Uuid,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub firebase_uid: String,
    pub firebase_email: String,
    pub stripe_customer_id: String,
}

pub struct NewUserRow {
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub firebase_uid: String,
    pub firebase_email: String,
    pub stripe_customer_id: String,
}

#[async_trait]
pub trait UserStore: Sync + Send {
    async fn add_user(&self, new_user: NewUserRow) -> Result<UserRow, Error>;

    async fn find_user_by_id(&self, user_id: Uuid) -> Result<Option<UserRow>, Error>;

    async fn find_user_by_firebase_uid(
        &self,
        firebase_uid: String,
    ) -> Result<Option<UserRow>, Error>;
}

#[async_trait]
impl UserStore for PgOnDemandStore {
    async fn add_user(&self, new_user: NewUserRow) -> Result<UserRow, Error> {
        Ok(sqlx::query_file_as!(
            UserRow,
            "queries/user/insert.sql",
            &new_user.create_time,
            &new_user.update_time,
            &new_user.firebase_uid,
            &new_user.firebase_email,
            &new_user.stripe_customer_id,
        )
        .fetch_one(&*self.pool)
        .await?)
    }

    async fn find_user_by_id(&self, user_id: Uuid) -> Result<Option<UserRow>, Error> {
        Ok(
            sqlx::query_file_as!(UserRow, "queries/user/find_by_id.sql", &user_id,)
                .fetch_optional(&*self.pool)
                .await?,
        )
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
        .fetch_optional(&*self.pool)
        .await?)
    }
}
