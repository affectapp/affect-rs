use crate::sqlx::client::PgDatabaseClient;
use anyhow::Context;
use lazy_static::lazy_static;
use testcontainers::{
    clients::Cli,
    images::generic::{GenericImage, WaitFor},
    Container, Docker,
};

lazy_static! {
    static ref DOCKER_CLI: Cli = Cli::default();
//     static ref PG_CONTAINER: AsyncOnce<Arc<PgContainer<'static>>> = AsyncOnce::new(async {
//         Arc::new(
//             PgContainer::start(&DOCKER_CLI)
//                 .await
//                 .expect("failed to start static PgContainer"),
//         )
//     });
}

pub struct PgContainer<'a> {
    /// The pool of connections to the pg container.
    pub pool: PgDatabaseClient,

    /// Owns container instance because when container is dropped, the
    /// container is stopped.
    #[allow(dead_code)]
    container: Container<'a, Cli, GenericImage>,
}

impl<'a> PgContainer<'a> {
    /// Starts a postgres instance via docker and opens a connection pool.
    pub async fn start() -> Result<PgContainer<'a>, anyhow::Error> {
        println!("Starting postgres container...");
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

        let container = DOCKER_CLI.run(generic_postgres);
        println!("Postgres container {} started", container.id());

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

        container.prepare().await?;

        Ok(container)
    }

    /// Run migrations and other setup.
    async fn prepare(&self) -> Result<(), anyhow::Error> {
        println!(
            "Runnning migrations on postgres container {}",
            self.container.id()
        );
        self.pool.run_migrations().await?;
        Ok(())
    }
}

impl<'a> Drop for PgContainer<'a> {
    fn drop(&mut self) {
        let id = self.container.id();
        self.container.stop();
        self.container.rm();
        println!("Postgres container {} stopped and removed", id);
    }
}
