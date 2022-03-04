use crate::page_token::PageTokenable;
use crate::{Error, PgPool};
use async_trait::async_trait;
use chrono::serde::ts_nanoseconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Debug, FromRow, PartialEq)]
pub struct AccountRow {
    pub account_id: Uuid,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub item_id: Uuid,
    pub plaid_account_id: String,
    pub name: String,
    pub mask: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewAccountRow {
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub item_id: Uuid,
    pub plaid_account_id: String,
    pub name: String,
    pub mask: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct AccountPageToken {
    #[serde(with = "ts_nanoseconds")]
    pub create_time: DateTime<Utc>,
}

impl PageTokenable<AccountPageToken> for AccountRow {
    fn page_token(&self) -> AccountPageToken {
        AccountPageToken {
            create_time: self.create_time.clone(),
        }
    }
}

#[async_trait]
pub trait AccountStore: Sync + Send {
    async fn add_account(&self, new_row: NewAccountRow) -> Result<AccountRow, Error>;

    async fn list_accounts(
        &self,
        page_size: i64,
        page_token: Option<AccountPageToken>,
    ) -> Result<Vec<AccountRow>, Error>;

    async fn count_accounts(&self) -> Result<i64, Error>;

    async fn list_and_count_accounts(
        &self,
        page_size: i64,
        page_token: Option<AccountPageToken>,
    ) -> Result<(Vec<AccountRow>, i64), Error> {
        let list_fut = self.list_accounts(page_size, page_token);
        let count_fut = self.count_accounts();
        futures::try_join!(list_fut, count_fut)
    }
}

pub struct PgAccountStore {
    pool: Arc<PgPool>,
}

impl PgAccountStore {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccountStore for PgAccountStore {
    async fn add_account(&self, new_row: NewAccountRow) -> Result<AccountRow, Error> {
        Ok(sqlx::query_file_as!(
            AccountRow,
            "queries/account/insert.sql",
            new_row.create_time,
            new_row.update_time,
            new_row.item_id,
            new_row.plaid_account_id,
            new_row.name,
            new_row.mask,
        )
        .fetch_one(self.pool.inner())
        .await?)
    }

    async fn list_accounts(
        &self,
        page_size: i64,
        page_token: Option<AccountPageToken>,
    ) -> Result<Vec<AccountRow>, Error> {
        let rows = match page_token {
            Some(page_token) => {
                // Query by page token:
                sqlx::query_file_as!(
                    AccountRow,
                    "queries/account/list_at_page.sql",
                    page_token.create_time,
                    page_size,
                )
                .fetch_all(self.pool.inner())
                .await?
            }
            None => {
                // Query first page:
                sqlx::query_file_as!(AccountRow, "queries/account/list.sql", page_size)
                    .fetch_all(self.pool.inner())
                    .await?
            }
        };
        Ok(rows)
    }

    async fn count_accounts(&self) -> Result<i64, Error> {
        Ok(sqlx::query_file!("queries/account/count.sql")
            .fetch_one(self.pool.inner())
            .await?
            .count
            .expect("null count query"))
    }
}
