use crate::{
    sqlx::store::{PgOnDemandStore, PgTransactionalStore},
    Error,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgExecutor};
use uuid::Uuid;

#[derive(Clone, Debug, FromRow, PartialEq)]
pub struct CauseRecipientRow {
    pub cause_id: Uuid,
    pub nonprofit_id: Uuid,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewCauseRecipientRow {
    pub cause_id: Uuid,
    pub nonprofit_id: Uuid,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[async_trait]
pub trait CauseRecipientStore: Sync + Send {
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
impl CauseRecipientStore for PgOnDemandStore {
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
impl<'a> CauseRecipientStore for PgTransactionalStore<'a> {
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
