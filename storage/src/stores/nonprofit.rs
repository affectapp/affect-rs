use crate::{page_token::PageTokenable, sqlx::store::PgOnDemandStore, Error};
use async_trait::async_trait;
use chrono::serde::ts_nanoseconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, FromRow, PartialEq)]
pub struct NonprofitRow {
    pub nonprofit_id: Uuid,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub change_nonprofit_id: String,
    pub icon_url: String,
    pub name: String,
    pub ein: String,
    pub mission: String,
    pub category: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewNonprofitRow {
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub change_nonprofit_id: String,
    pub icon_url: String,
    pub name: String,
    pub ein: String,
    pub mission: String,
    pub category: String,
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

#[async_trait]
pub trait NonprofitStore: Sync + Send {
    async fn add_nonprofit(&self, new_nonprofit: NewNonprofitRow) -> Result<NonprofitRow, Error>;

    async fn list_nonprofits(
        &self,
        page_size: i64,
        page_token: Option<NonprofitPageToken>,
    ) -> Result<Vec<NonprofitRow>, Error>;

    async fn count_nonprofits(&self) -> Result<i64, Error>;

    async fn list_and_count_nonprofits(
        &self,
        page_size: i64,
        page_token: Option<NonprofitPageToken>,
    ) -> Result<(Vec<NonprofitRow>, i64), Error> {
        let list_fut = self.list_nonprofits(page_size, page_token);
        let count_fut = self.count_nonprofits();
        futures::try_join!(list_fut, count_fut)
    }
}

#[async_trait]
impl NonprofitStore for PgOnDemandStore {
    async fn add_nonprofit(&self, new_profit: NewNonprofitRow) -> Result<NonprofitRow, Error> {
        Ok(sqlx::query_file_as!(
            NonprofitRow,
            "queries/nonprofit/insert.sql",
            &new_profit.create_time,
            &new_profit.update_time,
            &new_profit.change_nonprofit_id,
            &new_profit.icon_url,
            &new_profit.name,
            &new_profit.ein,
            &new_profit.mission,
            &new_profit.category,
        )
        .fetch_one(&*self.pool)
        .await?)
    }

    async fn list_nonprofits(
        &self,
        page_size: i64,
        page_token: Option<NonprofitPageToken>,
    ) -> Result<Vec<NonprofitRow>, Error> {
        let rows = match page_token {
            Some(page_token) => {
                // Query by page token:
                sqlx::query_file_as!(
                    NonprofitRow,
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
                sqlx::query_file_as!(NonprofitRow, "queries/nonprofit/list.sql", page_size)
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
            .count
            .expect("null count query"))
    }
}
