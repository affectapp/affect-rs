use crate::{sqlx::store::PgOnDemandStore, sqlx::store::PgTransactionalStore, Error};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{postgres::PgTypeInfo, FromRow, PgExecutor, Postgres};
use uuid::Uuid;

#[derive(Clone, Debug, FromRow)]
pub struct AffiliateRow {
    pub affiliate_id: Uuid,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub stripe_account_id: String,
    pub company_name: String,
    pub contact_email: String,
    pub business_type: BusinessType,
    pub asserted_nonprofit_id: Uuid,
}

#[derive(Clone, Debug, FromRow)]
pub struct AffiliateFullRow {
    pub affiliate_id: Uuid,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub stripe_account_id: String,
    pub company_name: String,
    pub contact_email: String,
    pub business_type: BusinessType,
    pub asserted_nonprofit_id: Uuid,
    pub affiliate_managers: AffiliateManagerRowVec,
}

#[derive(Clone, Debug, FromRow)]
pub struct NewAffiliateRow {
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub stripe_account_id: String,
    pub company_name: String,
    pub contact_email: String,
    pub business_type: BusinessType,
    pub asserted_nonprofit_id: Uuid,
}

#[derive(Clone, Debug, FromRow, sqlx::Type)]
pub struct AffiliateManagerRow {
    pub affiliate_id: Uuid,
    pub user_id: Uuid,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow)]
pub struct NewAffiliateManagerRow {
    pub affiliate_id: Uuid,
    pub user_id: Uuid,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(Clone, Debug, sqlx::Decode)]
pub struct AffiliateManagerRowVec(Vec<AffiliateManagerRow>);

impl AffiliateManagerRowVec {
    pub fn inner(self) -> Vec<AffiliateManagerRow> {
        self.0
    }
}

impl sqlx::Type<Postgres> for AffiliateManagerRowVec {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_affiliate_managers")
    }
}

#[derive(Clone, Debug, sqlx::Type)]
#[sqlx(type_name = "business_type", rename_all = "snake_case")]
pub enum BusinessType {
    Individual,
    Company,
    Nonprofit,
    GovernmentEntity,
}

#[async_trait]
pub trait AffiliateStore: Sync + Send {
    async fn add_affiliate(&self, new_row: NewAffiliateRow) -> Result<AffiliateRow, Error>;

    async fn find_affiliate_by_id(
        &self,
        affiliate_id: Uuid,
    ) -> Result<Option<AffiliateFullRow>, Error>;

    async fn add_affiliate_manager(
        &self,
        new_row: NewAffiliateManagerRow,
    ) -> Result<AffiliateManagerRow, Error>;

    async fn list_affiliate_managers_for_affilate(
        &self,
        affiliate_id: Uuid,
    ) -> Result<Vec<AffiliateManagerRow>, Error>;

    async fn list_affiliate_managers_for_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<AffiliateManagerRow>, Error>;
}

#[async_trait]
impl AffiliateStore for PgOnDemandStore {
    async fn add_affiliate(&self, new_row: NewAffiliateRow) -> Result<AffiliateRow, Error> {
        Ok(add_affiliate(&*self.pool, new_row).await?)
    }

    async fn find_affiliate_by_id(
        &self,
        affiliate_id: Uuid,
    ) -> Result<Option<AffiliateFullRow>, Error> {
        Ok(find_affiliate_by_id(&*self.pool, affiliate_id).await?)
    }

    async fn add_affiliate_manager(
        &self,
        new_row: NewAffiliateManagerRow,
    ) -> Result<AffiliateManagerRow, Error> {
        Ok(add_affiliate_manager(&*self.pool, new_row).await?)
    }

    async fn list_affiliate_managers_for_affilate(
        &self,
        affiliate_id: Uuid,
    ) -> Result<Vec<AffiliateManagerRow>, Error> {
        Ok(list_affiliate_managers_for_affilate(&*self.pool, affiliate_id).await?)
    }

    async fn list_affiliate_managers_for_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<AffiliateManagerRow>, Error> {
        Ok(list_affiliate_managers_for_user(&*self.pool, user_id).await?)
    }
}

#[async_trait]
impl<'a> AffiliateStore for PgTransactionalStore<'a> {
    async fn add_affiliate(&self, new_row: NewAffiliateRow) -> Result<AffiliateRow, Error> {
        let mut lock = self.txn.lock().await;
        Ok(add_affiliate(&mut *lock, new_row).await?)
    }

    async fn find_affiliate_by_id(
        &self,
        affiliate_id: Uuid,
    ) -> Result<Option<AffiliateFullRow>, Error> {
        let mut lock = self.txn.lock().await;
        Ok(find_affiliate_by_id(&mut *lock, affiliate_id).await?)
    }

    async fn add_affiliate_manager(
        &self,
        new_row: NewAffiliateManagerRow,
    ) -> Result<AffiliateManagerRow, Error> {
        let mut lock = self.txn.lock().await;
        Ok(add_affiliate_manager(&mut *lock, new_row).await?)
    }

    async fn list_affiliate_managers_for_affilate(
        &self,
        affiliate_id: Uuid,
    ) -> Result<Vec<AffiliateManagerRow>, Error> {
        let mut lock = self.txn.lock().await;
        Ok(list_affiliate_managers_for_affilate(&mut *lock, affiliate_id).await?)
    }

    async fn list_affiliate_managers_for_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<AffiliateManagerRow>, Error> {
        let mut lock = self.txn.lock().await;
        Ok(list_affiliate_managers_for_user(&mut *lock, user_id).await?)
    }
}

async fn add_affiliate<'a, E>(executor: E, new_row: NewAffiliateRow) -> Result<AffiliateRow, Error>
where
    E: PgExecutor<'a>,
{
    Ok(sqlx::query_file_as!(
        AffiliateRow,
        "queries/affiliate/insert.sql",
        new_row.create_time,
        new_row.update_time,
        new_row.stripe_account_id,
        new_row.company_name,
        new_row.contact_email,
        new_row.business_type as BusinessType,
        new_row.asserted_nonprofit_id,
    )
    .fetch_one(executor)
    .await?)
}

async fn find_affiliate_by_id<'a, E>(
    executor: E,
    affiliate_id: Uuid,
) -> Result<Option<AffiliateFullRow>, Error>
where
    E: PgExecutor<'a>,
{
    Ok(sqlx::query_file_as!(
        AffiliateFullRow,
        "queries/affiliate/find_by_id.sql",
        affiliate_id
    )
    .fetch_optional(executor)
    .await?)
}

async fn add_affiliate_manager<'a, E>(
    executor: E,
    new_row: NewAffiliateManagerRow,
) -> Result<AffiliateManagerRow, Error>
where
    E: PgExecutor<'a>,
{
    Ok(sqlx::query_file_as!(
        AffiliateManagerRow,
        "queries/affiliate_manager/insert.sql",
        new_row.affiliate_id,
        new_row.user_id,
        new_row.create_time,
        new_row.update_time,
    )
    .fetch_one(executor)
    .await?)
}

async fn list_affiliate_managers_for_affilate<'a, E>(
    executor: E,
    affiliate_id: Uuid,
) -> Result<Vec<AffiliateManagerRow>, Error>
where
    E: PgExecutor<'a>,
{
    Ok(sqlx::query_file_as!(
        AffiliateManagerRow,
        "queries/affiliate_manager/list_for_affiliate.sql",
        affiliate_id
    )
    .fetch_all(executor)
    .await?)
}

async fn list_affiliate_managers_for_user<'a, E>(
    executor: E,
    user_id: Uuid,
) -> Result<Vec<AffiliateManagerRow>, Error>
where
    E: PgExecutor<'a>,
{
    Ok(sqlx::query_file_as!(
        AffiliateManagerRow,
        "queries/affiliate_manager/list_for_user.sql",
        user_id
    )
    .fetch_all(executor)
    .await?)
}
