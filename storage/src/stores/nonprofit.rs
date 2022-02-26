use crate::{Error, PgPool};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
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

#[async_trait]
pub trait NonprofitStore: Sync + Send {
    async fn add_nonprofit(&self, new_user: NewNonprofitRow) -> Result<NonprofitRow, Error>;
}

pub struct PgNonprofitStore {
    pool: Arc<PgPool>,
}

impl PgNonprofitStore {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
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
}
