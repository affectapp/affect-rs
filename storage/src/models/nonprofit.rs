use crate::{models::affiliate::AffiliateRow, page_token::PageTokenable};
use chrono::{serde::ts_nanoseconds, DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgTypeInfo, FromRow, Postgres};
use uuid::Uuid;

#[derive(Clone, Debug, FromRow, PartialEq)]
pub struct NonprofitRow {
    pub nonprofit_id: Uuid,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub change_nonprofit_id: Option<String>,
    pub icon_url: String,
    pub name: String,
    pub ein: String,
    pub mission: String,
    pub category: String,
    pub affiliate_id: Option<Uuid>,
}

impl sqlx::Type<Postgres> for NonprofitRow {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("nonprofits")
    }
}

impl<'a> sqlx::decode::Decode<'a, sqlx::Postgres> for NonprofitRow {
    fn decode(
        value: sqlx::postgres::PgValueRef<'a>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let mut decoder = sqlx::postgres::types::PgRecordDecoder::new(value)?;
        let nonprofit_id = decoder.try_decode::<Uuid>()?;
        let create_time = decoder.try_decode::<DateTime<Utc>>()?;
        let update_time = decoder.try_decode::<DateTime<Utc>>()?;
        let change_nonprofit_id = decoder.try_decode::<Option<String>>()?;
        let icon_url = decoder.try_decode::<String>()?;
        let name = decoder.try_decode::<String>()?;
        let ein = decoder.try_decode::<String>()?;
        let mission = decoder.try_decode::<String>()?;
        let category = decoder.try_decode::<String>()?;
        let affiliate_id = decoder.try_decode::<Option<Uuid>>()?;
        Ok(NonprofitRow {
            nonprofit_id,
            create_time,
            update_time,
            change_nonprofit_id,
            icon_url,
            name,
            ein,
            mission,
            category,
            affiliate_id,
        })
    }
}

#[derive(Clone, Debug, FromRow, PartialEq)]
pub struct FullNonprofitRow {
    pub nonprofit: NonprofitRow,
    pub affiliate: Option<AffiliateRow>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewNonprofitRow {
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub change_nonprofit_id: Option<String>,
    pub icon_url: String,
    pub name: String,
    pub ein: String,
    pub mission: String,
    pub category: String,
    pub affiliate_id: Option<Uuid>,
}

#[derive(Serialize, Deserialize)]
pub struct NonprofitPageToken {
    #[serde(with = "ts_nanoseconds")]
    pub create_time: DateTime<Utc>,

    pub nonprofit_id: Uuid,
}

impl PageTokenable<NonprofitPageToken> for NonprofitRow {
    fn page_token(&self) -> NonprofitPageToken {
        NonprofitPageToken {
            create_time: self.create_time.clone(),
            nonprofit_id: self.nonprofit_id.clone(),
        }
    }
}

impl PageTokenable<NonprofitPageToken> for FullNonprofitRow {
    fn page_token(&self) -> NonprofitPageToken {
        self.nonprofit.page_token()
    }
}
