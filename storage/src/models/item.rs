use crate::page_token::PageTokenable;
use chrono::{serde::ts_nanoseconds, DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, FromRow, sqlx::Decode)]
pub struct ItemRow {
    pub item_id: Uuid,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub user_id: Uuid,
    pub plaid_item_id: String,
    pub plaid_access_token: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewItemRow {
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub user_id: Uuid,
    pub plaid_item_id: String,
    pub plaid_access_token: String,
}

#[derive(Serialize, Deserialize)]
pub struct ItemPageToken {
    #[serde(with = "ts_nanoseconds")]
    pub create_time: DateTime<Utc>,

    pub item_id: Uuid,
}

impl PageTokenable<ItemPageToken> for ItemRow {
    fn page_token(&self) -> ItemPageToken {
        ItemPageToken {
            create_time: self.create_time.clone(),
            item_id: self.item_id.clone(),
        }
    }
}
