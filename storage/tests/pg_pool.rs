use sqlx::Row;
use testcontainers::clients::Cli;

mod common;

#[tokio::test]
async fn pg_pool_provides_connection() -> Result<(), anyhow::Error> {
    let docker_cli = Cli::default();
    let pg_container = common::setup_pg_container(&docker_cli).await?;
    let pg_pool = pg_container.pool;

    let one: i32 = sqlx::query("SELECT 1")
        .fetch_one(pg_pool.inner())
        .await?
        .get(0);
    assert_eq!(one, 1);
    Ok(())
}
