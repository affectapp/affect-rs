use std::time::Duration;

use affect_status::Status;
use sqlx::{migrate::MigrateError, postgres::PgPoolOptions, Pool, Postgres};

pub mod page_token;
pub mod stores;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    // Error occurred when executing some SQL operation.
    #[error("sql failed: {0:?}")]
    Sql(#[from] sqlx::Error),

    // Error occurred when running migrations.
    #[error("migration failed: {0:?}")]
    Migrate(#[from] MigrateError),

    // Page token serialization/deserialization failed.
    #[error(transparent)]
    PageToken(anyhow::Error),

    // Some other/unexpected error occurred.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<Error> for Status {
    fn from(error: Error) -> Self {
        return Status::internal(format!("storage error: {:?}", error));
    }
}

pub struct PgPool {
    inner: Pool<Postgres>,
}

impl PgPool {
    // Connects to the provided postgres URI and returns the connected pool.
    pub async fn connect(postgres_uri: String) -> Result<Self, Error> {
        // let connect_options = PgConnectOptions::new();
        let inner = PgPoolOptions::new()
            .max_connections(2)
            .connect_timeout(Duration::from_secs(5))
            .connect(&postgres_uri)
            .await?;
        Ok(Self { inner })
    }

    // Returns reference to the underlying sqlx pg pool.
    pub fn inner(&self) -> &Pool<Postgres> {
        &self.inner
    }

    // Run migrations.
    pub async fn run_migrations(&self) -> Result<(), Error> {
        Ok(sqlx::migrate!().run(self.inner()).await?)
    }
}
