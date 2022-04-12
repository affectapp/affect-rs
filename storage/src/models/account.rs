use chrono::{DateTime, Utc};
use sqlx::FromRow;
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
    pub stripe_bank_account_id: String,
}

impl<'a> sqlx::decode::Decode<'a, sqlx::Postgres> for AccountRow {
    fn decode(
        value: sqlx::postgres::PgValueRef<'a>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let mut decoder = sqlx::postgres::types::PgRecordDecoder::new(value)?;
        let account_id = decoder.try_decode::<Uuid>()?;
        let create_time = decoder.try_decode::<DateTime<Utc>>()?;
        let update_time = decoder.try_decode::<DateTime<Utc>>()?;
        let item_id = decoder.try_decode::<Uuid>()?;
        let plaid_account_id = decoder.try_decode::<String>()?;
        let name = decoder.try_decode::<String>()?;
        let mask = decoder.try_decode::<Option<String>>()?;
        let stripe_bank_account_id = decoder.try_decode::<String>()?;
        Ok(AccountRow {
            account_id,
            create_time,
            update_time,
            item_id,
            plaid_account_id,
            name,
            mask,
            stripe_bank_account_id,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewAccountRow {
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub item_id: Uuid,
    pub plaid_account_id: String,
    pub name: String,
    pub mask: Option<String>,
    pub stripe_bank_account_id: String,
}
