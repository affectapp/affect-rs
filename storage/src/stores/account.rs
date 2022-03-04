use crate::{Error, PgPool};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
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

#[async_trait]
pub trait AccountStore: Sync + Send {
    async fn add_account(&self, new_row: NewAccountRow) -> Result<AccountRow, Error>;

    async fn list_accounts_for_item(&self, item_id: Uuid) -> Result<Vec<AccountRow>, Error>;
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

    async fn list_accounts_for_item(&self, item_id: Uuid) -> Result<Vec<AccountRow>, Error> {
        let rows = sqlx::query_file_as!(AccountRow, "queries/account/list_for_item.sql", item_id)
            .fetch_all(self.pool.inner())
            .await?;
        Ok(rows)
    }
}
