use crate::{
    page_token::PageTokenable, sqlx::store::PgOnDemandStore, stores::affiliate::AffiliateRow, Error,
};
use async_trait::async_trait;
use chrono::serde::ts_nanoseconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgTypeInfo;
use sqlx::{FromRow, Postgres};
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

#[async_trait]
pub trait NonprofitStore: Sync + Send {
    async fn add_nonprofit(&self, new_nonprofit: NewNonprofitRow) -> Result<NonprofitRow, Error>;

    async fn find_nonprofit_by_id(
        &self,
        nonprofit_id: Uuid,
    ) -> Result<Option<FullNonprofitRow>, Error>;

    async fn list_nonprofits(
        &self,
        page_size: i64,
        page_token: Option<NonprofitPageToken>,
    ) -> Result<Vec<FullNonprofitRow>, Error>;

    async fn count_nonprofits(&self) -> Result<i64, Error>;

    async fn list_and_count_nonprofits(
        &self,
        page_size: i64,
        page_token: Option<NonprofitPageToken>,
    ) -> Result<(Vec<FullNonprofitRow>, i64), Error> {
        let list_fut = self.list_nonprofits(page_size, page_token);
        let count_fut = self.count_nonprofits();
        futures::try_join!(list_fut, count_fut)
    }

    async fn list_nonprofits_by_search(
        &self,
        page_size: i64,
        page_token: Option<NonprofitPageToken>,
        query: &str,
    ) -> Result<Vec<FullNonprofitRow>, Error>;

    async fn count_nonprofits_by_search(&self, query: &str) -> Result<i64, Error>;

    async fn list_and_count_nonprofits_by_search(
        &self,
        page_size: i64,
        page_token: Option<NonprofitPageToken>,
        query: &str,
    ) -> Result<(Vec<FullNonprofitRow>, i64), Error> {
        let list_fut = self.list_nonprofits_by_search(page_size, page_token, query);
        let count_fut = self.count_nonprofits_by_search(query);
        futures::try_join!(list_fut, count_fut)
    }
}

#[async_trait]
impl NonprofitStore for PgOnDemandStore {
    async fn add_nonprofit(&self, new_nonprofit: NewNonprofitRow) -> Result<NonprofitRow, Error> {
        Ok(sqlx::query_file_as!(
            NonprofitRow,
            "queries/nonprofit/insert.sql",
            new_nonprofit.create_time,
            new_nonprofit.update_time,
            new_nonprofit.change_nonprofit_id,
            new_nonprofit.icon_url,
            new_nonprofit.name,
            new_nonprofit.ein,
            new_nonprofit.mission,
            new_nonprofit.category,
            new_nonprofit.affiliate_id,
        )
        .fetch_one(&*self.pool)
        .await?)
    }

    async fn find_nonprofit_by_id(
        &self,
        nonprofit_id: Uuid,
    ) -> Result<Option<FullNonprofitRow>, Error> {
        Ok(sqlx::query_file_as!(
            FullNonprofitRow,
            "queries/nonprofit/find_by_id.sql",
            nonprofit_id
        )
        .fetch_optional(&*self.pool)
        .await?)
    }

    async fn list_nonprofits(
        &self,
        page_size: i64,
        page_token: Option<NonprofitPageToken>,
    ) -> Result<Vec<FullNonprofitRow>, Error> {
        let rows = match page_token {
            Some(page_token) => {
                // Query by page token:
                sqlx::query_file_as!(
                    FullNonprofitRow,
                    "queries/nonprofit/list_at_page.sql",
                    page_token.create_time,
                    page_token.nonprofit_id,
                    page_size,
                )
                .fetch_all(&*self.pool)
                .await?
            }
            None => {
                // Query first page:
                sqlx::query_file_as!(FullNonprofitRow, "queries/nonprofit/list.sql", page_size)
                    .fetch_all(&*self.pool)
                    .await?
            }
        };
        Ok(rows)
    }

    async fn count_nonprofits(&self) -> Result<i64, Error> {
        Ok(sqlx::query_file!("queries/nonprofit/count.sql")
            .fetch_one(&*self.pool)
            .await?
            .count)
    }

    async fn list_nonprofits_by_search(
        &self,
        page_size: i64,
        page_token: Option<NonprofitPageToken>,
        query: &str,
    ) -> Result<Vec<FullNonprofitRow>, Error> {
        let rows = match page_token {
            Some(page_token) => {
                // Query by page token:
                sqlx::query_file_as!(
                    FullNonprofitRow,
                    "queries/nonprofit/list_by_search_at_page.sql",
                    query,
                    page_token.create_time,
                    page_token.nonprofit_id,
                    page_size,
                )
                .fetch_all(&*self.pool)
                .await?
            }
            None => {
                // Query first page:
                sqlx::query_file_as!(
                    FullNonprofitRow,
                    "queries/nonprofit/list_by_search.sql",
                    query,
                    page_size
                )
                .fetch_all(&*self.pool)
                .await?
            }
        };
        Ok(rows)
    }

    async fn count_nonprofits_by_search(&self, query: &str) -> Result<i64, Error> {
        Ok(
            sqlx::query_file!("queries/nonprofit/count_by_search.sql", query)
                .fetch_one(&*self.pool)
                .await?
                .count,
        )
    }
}
