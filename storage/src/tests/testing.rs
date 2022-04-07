use crate::sqlx::client::PgDatabaseClient;
use anyhow::Context;
use testcontainers::{
    clients::Cli,
    images::generic::{GenericImage, WaitFor},
    Container, Docker,
};

pub struct PgContainer<'a> {
    pub pool: PgDatabaseClient,

    // Owns container instance because when container is dropped, the
    // container is stopped.
    #[allow(dead_code)]
    container: Container<'a, Cli, GenericImage>,
}

// Starts a postgres instance via docker and opens a connection pool.
pub async fn setup_pg_container<'a>(docker_cli: &'a Cli) -> Result<PgContainer<'a>, anyhow::Error> {
    let db = "postgres-db-test";
    let user = "postgres-user-test";
    let password = "postgres-password-test";

    let generic_postgres = GenericImage::new("postgres:14-alpine")
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_DB", db)
        .with_env_var("POSTGRES_USER", user)
        .with_env_var("POSTGRES_PASSWORD", password);

    let container = docker_cli.run(generic_postgres);

    let postgres_uri = format!(
        "postgres://{}:{}@localhost:{}/{}?sslmode=disable",
        user,
        password,
        container
            .get_host_port(5432)
            .context("failed getting postgres container port")?,
        db
    );

    let container = PgContainer {
        pool: PgDatabaseClient::connect(postgres_uri).await?,
        container,
    };

    // Run migrations to setup initial tables:
    container.pool.run_migrations().await?;

    Ok(container)
}
