use crate::{models::nonprofit::*, sqlx::store::PgOnDemandStore, Error};
use async_trait::async_trait;
use uuid::Uuid;

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
