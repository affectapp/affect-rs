use crate::{page_token::PageTokenable, sqlx::store::PgOnDemandStore, Error};
use async_trait::async_trait;
use chrono::serde::ts_nanoseconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, FromRow, PartialEq)]
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

#[async_trait]
pub trait ItemStore: Sync + Send {
    async fn add_item(&self, new_row: NewItemRow) -> Result<ItemRow, Error>;

    async fn list_items_for_user(
        &self,
        page_size: i64,
        page_token: Option<ItemPageToken>,
        user_id: Uuid,
    ) -> Result<Vec<ItemRow>, Error>;

    async fn count_items_for_user(&self, user_id: Uuid) -> Result<i64, Error>;

    async fn list_and_count_items_for_user(
        &self,
        page_size: i64,
        page_token: Option<ItemPageToken>,
        user_id: Uuid,
    ) -> Result<(Vec<ItemRow>, i64), Error> {
        let list_fut = self.list_items_for_user(page_size, page_token, user_id);
        let count_fut = self.count_items_for_user(user_id);
        futures::try_join!(list_fut, count_fut)
    }
}

#[async_trait]
impl ItemStore for PgOnDemandStore {
    async fn add_item(&self, new_row: NewItemRow) -> Result<ItemRow, Error> {
        Ok(sqlx::query_file_as!(
            ItemRow,
            "queries/item/insert.sql",
            new_row.create_time,
            new_row.update_time,
            new_row.user_id,
            new_row.plaid_item_id,
            new_row.plaid_access_token
        )
        .fetch_one(&*self.pool)
        .await?)
    }

    async fn list_items_for_user(
        &self,
        page_size: i64,
        page_token: Option<ItemPageToken>,
        user_id: Uuid,
    ) -> Result<Vec<ItemRow>, Error> {
        let rows = match page_token {
            Some(page_token) => {
                // Query by page token:
                sqlx::query_file_as!(
                    ItemRow,
                    "queries/item/list_at_page_for_user.sql",
                    page_token.create_time,
                    page_token.item_id,
                    &user_id,
                    page_size,
                )
                .fetch_all(&*self.pool)
                .await?
            }
            None => {
                // Query first page:
                sqlx::query_file_as!(
                    ItemRow,
                    "queries/item/list_for_user.sql",
                    page_size,
                    &user_id
                )
                .fetch_all(&*self.pool)
                .await?
            }
        };
        Ok(rows)
    }

    async fn count_items_for_user(&self, user_id: Uuid) -> Result<i64, Error> {
        Ok(
            sqlx::query_file!("queries/item/count_for_user.sql", &user_id)
                .fetch_one(&*self.pool)
                .await?
                .count
                .expect("null count query"),
        )
    }
}
