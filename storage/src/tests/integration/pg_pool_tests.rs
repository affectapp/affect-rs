use crate::tests::integration::containers::PgContainer;
use sqlx::Row;

#[tokio::test]
async fn pg_pool_provides_connection() -> Result<(), anyhow::Error> {
    let container = PgContainer::start().await?;
    let one: i32 = sqlx::query("SELECT 1")
        .fetch_one(container.pool.inner())
        .await?
        .get(0);
    assert_eq!(one, 1);
    Ok(())
}
