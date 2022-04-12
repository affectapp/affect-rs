use crate::page_token::PageTokenable;
use chrono::{serde::ts_nanoseconds, DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, FromRow, sqlx::Decode)]
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

#[derive(Serialize, Deserialize)]
pub struct UserPageToken {
    #[serde(with = "ts_nanoseconds")]
    pub create_time: DateTime<Utc>,
    pub user_id: Uuid,
}

impl PageTokenable<UserPageToken> for UserRow {
    fn page_token(&self) -> UserPageToken {
        UserPageToken {
            create_time: self.create_time.clone(),
            user_id: self.user_id.clone(),
        }
    }
}
