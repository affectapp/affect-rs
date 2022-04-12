use crate::models::nonprofit::NonprofitRow;
use chrono::{DateTime, Utc};
use sqlx::{postgres::PgTypeInfo, FromRow, Postgres};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, FromRow, sqlx::Decode)]
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

impl sqlx::Type<Postgres> for AffiliateRow {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("affiliates")
    }
}

#[derive(Clone, Debug, PartialEq, FromRow, sqlx::Decode)]
pub struct FullAffiliateRow {
    pub affiliate: AffiliateRow,
    pub asserted_nonprofit: Option<NonprofitRow>,
    pub affiliate_managers: AffiliateManagerRowVec,
}

impl sqlx::Type<Postgres> for FullAffiliateRow {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("full_affiliates")
    }
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

#[derive(Clone, Debug, PartialEq, FromRow, sqlx::Type)]
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

#[derive(Clone, Debug, PartialEq, sqlx::Decode)]
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

#[derive(Clone, Debug, PartialEq, sqlx::Type)]
#[sqlx(type_name = "business_type", rename_all = "snake_case")]
pub enum BusinessType {
    Individual,
    Company,
    Nonprofit,
    GovernmentEntity,
}
