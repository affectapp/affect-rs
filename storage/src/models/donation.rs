use chrono::{DateTime, Utc};
use sqlx::FromRow;
use sqlx::Type;
use uuid::Uuid;

#[derive(Clone, Debug, FromRow, PartialEq)]
pub struct DonationRow {
    pub donation_id: Uuid,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub nonprofit_id: Uuid,
    pub user_id: Uuid,
    pub affiliate_id: Option<Uuid>,
    pub currency_code: CurrencyCode,
    pub amount_units: i64,
    pub amount_nanos: i32,
}

#[derive(Clone, Debug, Type, PartialEq)]
#[sqlx(type_name = "currency_code", rename_all = "lowercase")]
pub enum CurrencyCode {
    USD,
}
