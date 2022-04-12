use crate::page_token::PageTokenable;
use chrono::{serde::ts_nanoseconds, DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgTypeInfo, FromRow, Postgres};
use uuid::Uuid;

#[derive(Clone, Debug, FromRow, sqlx::Decode)]
pub struct CauseRow {
    pub cause_id: Uuid,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub user_id: Uuid,
    pub name: String,
}

impl sqlx::Type<Postgres> for CauseRow {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("causes")
    }
}

#[derive(Clone, Debug, FromRow)]
pub struct FullCauseRow {
    pub cause: CauseRow,
    pub cause_recipients: CauseRecipientRowVec,
}

#[derive(Clone, Debug)]
pub struct NewCauseRow {
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub user_id: Uuid,
    pub name: String,
}

#[derive(Clone, Debug, FromRow, sqlx::Decode)]
pub struct CauseRecipientRow {
    pub cause_id: Uuid,
    pub nonprofit_id: Uuid,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

impl sqlx::Type<Postgres> for CauseRecipientRow {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("cause_recipients")
    }
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

impl PageTokenable<CausePageToken> for FullCauseRow {
    fn page_token(&self) -> CausePageToken {
        self.cause.page_token()
    }
}
