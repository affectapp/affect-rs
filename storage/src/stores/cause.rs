use crate::database::store::TransactionalStore;
use crate::page_token::PageTokenable;
use crate::sqlx::store::{PgOnDemandStore, PgTransactionalStore};
use crate::Error;
use async_trait::async_trait;
use chrono::serde::ts_nanoseconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgTypeInfo;
use sqlx::{FromRow, PgExecutor, Postgres};
use uuid::Uuid;

#[derive(Clone, Debug, FromRow, sqlx::Decode)]
pub struct CauseRow {
    pub cause_id: Uuid,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub user_id: Uuid,
    pub name: String,
}

#[derive(Clone, Debug, FromRow)]
pub struct CauseFullRow {
    pub cause_id: Uuid,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub user_id: Uuid,
    pub name: String,
    pub recipients: CauseRecipientRowVec,
}

#[derive(Clone, Debug)]
pub struct NewCauseRow {
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub user_id: Uuid,
    pub name: String,
}

#[derive(Clone, Debug, FromRow, sqlx::Type)]
pub struct CauseRecipientRow {
    pub cause_id: Uuid,
    pub nonprofit_id: Uuid,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct NewCauseRecipientRow {
    pub cause_id: Uuid,
    pub nonprofit_id: Uuid,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(Clone, Debug, sqlx::Decode)]
pub struct CauseRecipientRowVec(Vec<CauseRecipientRow>);

impl CauseRecipientRowVec {
    pub fn inner(self) -> Vec<CauseRecipientRow> {
        self.0
    }
}

impl sqlx::Type<Postgres> for CauseRecipientRowVec {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_cause_recipients")
    }
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

impl PageTokenable<CausePageToken> for CauseFullRow {
    fn page_token(&self) -> CausePageToken {
        CausePageToken {
            create_time: self.create_time.clone(),
            cause_id: self.cause_id.clone(),
        }
    }
}

#[async_trait]
pub trait CauseStore: Send + Sync {
    async fn add_cause(&self, new_row: NewCauseRow) -> Result<CauseRow, Error>;

    async fn list_causes_for_user(
        &self,
        page_size: i64,
        page_token: Option<CausePageToken>,
        user_id: Uuid,
    ) -> Result<Vec<CauseFullRow>, Error>;

    async fn count_causes_for_user(&self, user_id: Uuid) -> Result<i64, Error>;

    async fn list_and_count_causes_for_user(
        &self,
        page_size: i64,
        page_token: Option<CausePageToken>,
        user_id: Uuid,
    ) -> Result<(Vec<CauseFullRow>, i64), Error> {
        let list_fut = self.list_causes_for_user(page_size, page_token, user_id);
        let count_fut = self.count_causes_for_user(user_id);
        futures::try_join!(list_fut, count_fut)
    }

    async fn add_cause_recipient(
        &self,
        new_row: NewCauseRecipientRow,
    ) -> Result<CauseRecipientRow, Error>;

    async fn list_cause_recipients_for_cause(
        &self,
        cause_id: Uuid,
    ) -> Result<Vec<CauseRecipientRow>, Error>;
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
    ) -> Result<Vec<CauseFullRow>, Error> {
        Ok(list_causes_for_user(&*self.pool, page_size, page_token, user_id).await?)
    }

    async fn count_causes_for_user(&self, user_id: Uuid) -> Result<i64, Error> {
        Ok(count_causes_for_user(&*self.pool, user_id).await?)
    }

    async fn add_cause_recipient(
        &self,
        new_row: NewCauseRecipientRow,
    ) -> Result<CauseRecipientRow, Error> {
        Ok(add_cause_recipient(&*self.pool, new_row).await?)
    }

    async fn list_cause_recipients_for_cause(
        &self,
        cause_id: Uuid,
    ) -> Result<Vec<CauseRecipientRow>, Error> {
        Ok(list_cause_recipients_for_cause(&*self.pool, cause_id).await?)
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
    ) -> Result<Vec<CauseFullRow>, Error> {
        let mut lock = self.txn.lock().await;
        Ok(list_causes_for_user(&mut *lock, page_size, page_token, user_id).await?)
    }

    async fn count_causes_for_user(&self, user_id: Uuid) -> Result<i64, Error> {
        let mut lock = self.txn.lock().await;
        Ok(count_causes_for_user(&mut *lock, user_id).await?)
    }

    async fn add_cause_recipient(
        &self,
        new_row: NewCauseRecipientRow,
    ) -> Result<CauseRecipientRow, Error> {
        let mut lock = self.txn.lock().await;
        Ok(add_cause_recipient(&mut *lock, new_row).await?)
    }

    async fn list_cause_recipients_for_cause(
        &self,
        cause_id: Uuid,
    ) -> Result<Vec<CauseRecipientRow>, Error> {
        let mut lock = self.txn.lock().await;
        Ok(list_cause_recipients_for_cause(&mut *lock, cause_id).await?)
    }
}

#[async_trait]
pub trait CauseAndRecipientStore {
    /// Adds a cause and recipients of the cause.
    async fn add_cause_and_recipients(
        &self,
        user_id: Uuid,
        recipient_nonprofit_ids: Vec<Uuid>,
    ) -> Result<(CauseRow, Vec<CauseRecipientRow>), Error>;
}

/// Implementation of the store for transactions.
#[async_trait]
impl<S> CauseAndRecipientStore for S
where
    S: CauseStore + TransactionalStore,
{
    async fn add_cause_and_recipients(
        &self,
        user_id: Uuid,
        recipient_nonprofit_ids: Vec<Uuid>,
    ) -> Result<(CauseRow, Vec<CauseRecipientRow>), Error> {
        let now = Utc::now();
        let cause_row = self
            .add_cause(NewCauseRow {
                create_time: now,
                update_time: now,
                user_id,
                name: "some name".to_string(),
            })
            .await?;

        let mut recipient_rows = Vec::new();
        for recipient_nonprofit_id in recipient_nonprofit_ids {
            recipient_rows.push(
                self.add_cause_recipient(NewCauseRecipientRow {
                    cause_id: cause_row.cause_id.clone(),
                    nonprofit_id: recipient_nonprofit_id.clone(),
                    create_time: now,
                    update_time: now,
                })
                .await?,
            );
        }
        Ok((cause_row, recipient_rows))
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
) -> Result<Vec<CauseFullRow>, Error>
where
    E: PgExecutor<'a>,
{
    let rows = match page_token {
        Some(page_token) => {
            // Query by page token:
            sqlx::query_file_as!(
                CauseFullRow,
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
                CauseFullRow,
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

async fn add_cause_recipient<'a, E>(
    executor: E,
    new_row: NewCauseRecipientRow,
) -> Result<CauseRecipientRow, Error>
where
    E: PgExecutor<'a>,
{
    Ok(sqlx::query_file_as!(
        CauseRecipientRow,
        "queries/cause_recipient/insert.sql",
        new_row.cause_id,
        new_row.nonprofit_id,
        new_row.create_time,
        new_row.update_time,
    )
    .fetch_one(executor)
    .await?)
}

async fn list_cause_recipients_for_cause<'a, E>(
    executor: E,
    cause_id: Uuid,
) -> Result<Vec<CauseRecipientRow>, Error>
where
    E: PgExecutor<'a>,
{
    let rows = sqlx::query_file_as!(
        CauseRecipientRow,
        "queries/cause_recipient/list_for_cause.sql",
        &cause_id,
    )
    .fetch_all(executor)
    .await?;
    Ok(rows)
}
