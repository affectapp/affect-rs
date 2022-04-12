use crate::{
    models::affiliate::*, sqlx::store::PgOnDemandStore, sqlx::store::PgTransactionalStore, Error,
};
use async_trait::async_trait;
use sqlx::PgExecutor;
use uuid::Uuid;

#[async_trait]
pub trait AffiliateStore: Sync + Send {
    async fn add_affiliate(&self, new_row: NewAffiliateRow) -> Result<AffiliateRow, Error>;

    async fn find_affiliate_by_id(
        &self,
        affiliate_id: Uuid,
    ) -> Result<Option<FullAffiliateRow>, Error>;

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
    ) -> Result<Option<FullAffiliateRow>, Error> {
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
    ) -> Result<Option<FullAffiliateRow>, Error> {
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
) -> Result<Option<FullAffiliateRow>, Error>
where
    E: PgExecutor<'a>,
{
    Ok(sqlx::query_file_as!(
        FullAffiliateRow,
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
