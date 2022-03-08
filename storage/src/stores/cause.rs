use crate::page_token::PageTokenable;
use crate::sqlx::store::{PgOnDemandStore, PgTransactionalStore};
use crate::Error;
use async_trait::async_trait;
use chrono::serde::ts_nanoseconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgExecutor};
use uuid::Uuid;

#[derive(Clone, Debug, FromRow, PartialEq)]
pub struct CauseRow {
    pub cause_id: Uuid,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub user_id: Uuid,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewCauseRow {
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub user_id: Uuid,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct CausePageToken {
    #[serde(with = "ts_nanoseconds")]
    pub create_time: DateTime<Utc>,

    pub cause_id: Uuid,
}

impl PageTokenable<CausePageToken> for CauseRow {
    fn page_token(&self) -> CausePageToken {
        CausePageToken {
            create_time: self.create_time.clone(),
            cause_id: self.cause_id.clone(),
        }
    }
}

// #[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait CauseStore /* : Sync + Send */ {
    async fn add_cause(&self, new_row: NewCauseRow) -> Result<CauseRow, Error>;

    async fn list_causes_for_user(
        &self,
        page_size: i64,
        page_token: Option<CausePageToken>,
        user_id: Uuid,
    ) -> Result<Vec<CauseRow>, Error>;

    async fn count_causes_for_user(&self, user_id: Uuid) -> Result<i64, Error>;

    async fn list_and_count_causes_for_user(
        &self,
        page_size: i64,
        page_token: Option<CausePageToken>,
        user_id: Uuid,
    ) -> Result<(Vec<CauseRow>, i64), Error> {
        let list_fut = self.list_causes_for_user(page_size, page_token, user_id);
        let count_fut = self.count_causes_for_user(user_id);
        futures::try_join!(list_fut, count_fut)
    }
}

#[async_trait]
impl<'a> CauseStore for PgTransactionalStore<'a> {
    async fn add_cause(&self, new_row: NewCauseRow) -> Result<CauseRow, Error> {
        let mut lock = self.txn.lock().await;
        Ok(add_cause(&mut *lock, new_row).await?)
    }

    async fn list_causes_for_user(
        &self,
        page_size: i64,
        page_token: Option<CausePageToken>,
        user_id: Uuid,
    ) -> Result<Vec<CauseRow>, Error> {
        let mut lock = self.txn.lock().await;
        Ok(list_causes_for_user(&mut *lock, page_size, page_token, user_id).await?)
    }

    async fn count_causes_for_user(&self, user_id: Uuid) -> Result<i64, Error> {
        let mut lock = self.txn.lock().await;
        Ok(count_causes_for_user(&mut *lock, user_id).await?)
    }
}

#[async_trait]
impl CauseStore for PgOnDemandStore {
    async fn add_cause(&self, new_row: NewCauseRow) -> Result<CauseRow, Error> {
        Ok(add_cause(&*self.pool, new_row).await?)
    }

    async fn list_causes_for_user(
        &self,
        page_size: i64,
        page_token: Option<CausePageToken>,
        user_id: Uuid,
    ) -> Result<Vec<CauseRow>, Error> {
        Ok(list_causes_for_user(&*self.pool, page_size, page_token, user_id).await?)
    }

    async fn count_causes_for_user(&self, user_id: Uuid) -> Result<i64, Error> {
        Ok(count_causes_for_user(&*self.pool, user_id).await?)
    }
}

async fn add_cause<'a, E>(executor: E, new_row: NewCauseRow) -> Result<CauseRow, Error>
where
    E: PgExecutor<'a>,
{
    Ok(sqlx::query_file_as!(
        CauseRow,
        "queries/cause/insert.sql",
        new_row.create_time,
        new_row.update_time,
        new_row.user_id,
        new_row.name,
    )
    .fetch_one(executor)
    .await?)
}

async fn list_causes_for_user<'a, E>(
    executor: E,
    page_size: i64,
    page_token: Option<CausePageToken>,
    user_id: Uuid,
) -> Result<Vec<CauseRow>, Error>
where
    E: PgExecutor<'a>,
{
    let rows = match page_token {
        Some(page_token) => {
            // Query by page token:
            sqlx::query_file_as!(
                CauseRow,
                "queries/cause/list_at_page_for_user.sql",
                page_token.create_time,
                page_token.cause_id,
                &user_id,
                page_size,
            )
            .fetch_all(executor)
            .await?
        }
        None => {
            // Query first page:
            sqlx::query_file_as!(
                CauseRow,
                "queries/cause/list_for_user.sql",
                page_size,
                &user_id
            )
            .fetch_all(executor)
            .await?
        }
    };
    Ok(rows)
}

async fn count_causes_for_user<'a, E>(executor: E, user_id: Uuid) -> Result<i64, Error>
where
    E: PgExecutor<'a>,
{
    Ok(
        sqlx::query_file!("queries/cause/count_for_user.sql", &user_id)
            .fetch_one(executor)
            .await?
            .count
            .expect("null count query"),
    )
}
