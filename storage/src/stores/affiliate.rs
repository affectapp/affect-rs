use crate::{sqlx::store::PgOnDemandStore, Error};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
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
}

pub struct NewAffiliateRow {
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub stripe_account_id: String,
    pub company_name: String,
    pub contact_email: String,
    pub business_type: BusinessType,
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

    async fn find_affiliate_by_id(&self, affiliate_id: Uuid)
        -> Result<Option<AffiliateRow>, Error>;
}

#[async_trait]
impl AffiliateStore for PgOnDemandStore {
    async fn add_affiliate(&self, new_row: NewAffiliateRow) -> Result<AffiliateRow, Error> {
        Ok(sqlx::query_file_as!(
            AffiliateRow,
            "queries/affiliate/insert.sql",
            new_row.create_time,
            new_row.update_time,
            new_row.stripe_account_id,
            new_row.company_name,
            new_row.contact_email,
            new_row.business_type as BusinessType,
        )
        .fetch_one(&*self.pool)
        .await?)
    }

    async fn find_affiliate_by_id(
        &self,
        affiliate_id: Uuid,
    ) -> Result<Option<AffiliateRow>, Error> {
        Ok(sqlx::query_file_as!(
            AffiliateRow,
            "queries/affiliate/find_by_id.sql",
            affiliate_id
        )
        .fetch_optional(&*self.pool)
        .await?)
    }
}
