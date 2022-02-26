use affect_storage::PgPool;
use testcontainers::{
    clients::Cli,
    core::Port,
    images::generic::{GenericImage, WaitFor},
    Container, Docker, RunArgs,
};

pub struct PgContainer<'a> {
    pub pool: PgPool,

    // Owns container instance because when container is dropped, the
    // container is stopped.
    container: Container<'a, Cli, GenericImage>,
}

// Starts a postgres instance via docker and opens a connection pool.
pub async fn setup_pg_container<'a>(
    docker_cli: &'a Cli,
) -> Result<PgContainer<'a>, Box<dyn std::error::Error>> {
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
            .ok_or("failed getting host port")?,
        db
    );

    let container = PgContainer {
        pool: PgPool::connect(postgres_uri).await?,
        container,
    };

    // Run migrations to setup initial tables:
    container.pool.run_migrations().await?;

    Ok(container)
}
