use crate::page_token::PageTokenable;
use crate::{Error, PgPool};
use async_trait::async_trait;
use chrono::serde::ts_nanoseconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Debug, FromRow, PartialEq)]
pub struct NonprofitRow {
    pub nonprofit_id: Uuid,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub change_nonprofit_id: String,
    pub icon_url: String,
    pub title: String,
    pub ein: String,
    pub mission: String,
    pub category: String,
}

pub struct NewNonprofitRow {
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub change_nonprofit_id: String,
    pub icon_url: String,
    pub title: String,
    pub ein: String,
    pub mission: String,
    pub category: String,
}

#[derive(Serialize, Deserialize)]
pub struct NonprofitPageToken {
    #[serde(with = "ts_nanoseconds")]
    pub create_time: DateTime<Utc>,
}

impl PageTokenable<NonprofitPageToken> for NonprofitRow {
    fn page_token(&self) -> NonprofitPageToken {
        NonprofitPageToken {
            create_time: self.create_time.clone(),
        }
    }
}

#[async_trait]
pub trait NonprofitStore: Sync + Send {
    async fn add_nonprofit(&self, new_nonprofit: NewNonprofitRow) -> Result<NonprofitRow, Error>;

    async fn list_and_count_nonprofits(
        &self,
        page_size: i64,
        page_token: Option<NonprofitPageToken>,
    ) -> Result<(Vec<NonprofitRow>, i64), Error>;
}

pub struct PgNonprofitStore {
    pool: Arc<PgPool>,
}

impl PgNonprofitStore {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    async fn _list_nonprofits(
        &self,
        page_size: i64,
        page_token: Option<NonprofitPageToken>,
    ) -> Result<Vec<NonprofitRow>, Error> {
        match page_token {
            Some(page_token) => Ok(sqlx::query_as(
                "SELECT * \
                FROM nonprofits \
                WHERE (create_time) >= ($1) \
                ORDER BY create_time ASC \
                LIMIT $2",
            )
            .bind(page_token.create_time)
            .bind(page_size)
            .fetch_all(self.pool.inner())
            .await?),
            None => Ok(sqlx::query_as(
                "SELECT * \
                FROM nonprofits \
                ORDER BY create_time ASC \
                LIMIT $1",
            )
            .bind(page_size)
            .fetch_all(self.pool.inner())
            .await?),
        }
    }

    async fn _count_nonprofits(&self) -> Result<i64, Error> {
        Ok(sqlx::query(
            "SELECT COUNT(*) \
            FROM nonprofits",
        )
        .fetch_one(self.pool.inner())
        .await?
        .try_get(0)?)
    }
}

#[async_trait]
impl NonprofitStore for PgNonprofitStore {
    async fn add_nonprofit(&self, new_profit: NewNonprofitRow) -> Result<NonprofitRow, Error> {
        Ok(sqlx::query_as(
            r#"
            INSERT INTO nonprofits (
                create_time, 
                update_time, 
                change_nonprofit_id,
                icon_url,
                title,
                ein,
                mission,
                category
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *"#,
        )
        .bind(&new_profit.create_time)
        .bind(&new_profit.update_time)
        .bind(&new_profit.change_nonprofit_id)
        .bind(&new_profit.icon_url)
        .bind(&new_profit.title)
        .bind(&new_profit.ein)
        .bind(&new_profit.mission)
        .bind(&new_profit.category)
        .fetch_one(self.pool.inner())
        .await?)
    }

    async fn list_and_count_nonprofits(
        &self,
        page_size: i64,
        page_token: Option<NonprofitPageToken>,
    ) -> Result<(Vec<NonprofitRow>, i64), Error> {
        let list_fut = self._list_nonprofits(page_size, page_token);
        let count_fut = self._count_nonprofits();
        futures::try_join!(list_fut, count_fut)
    }
}
