use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, FromRow, PartialEq)]
pub struct DonationRow {
    pub donation_id: Uuid,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub nonprofit_id: Uuid,
    pub user_id: Uuid,
    pub affiliate_id: Option<Uuid>,
}
