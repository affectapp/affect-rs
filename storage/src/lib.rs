use affect_status::Status;
use sqlx::{migrate::MigrateError, postgres::PgPoolOptions, Pool, Postgres};

pub mod user;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("sql failed: {0:?}")]
    Sql(#[from] sqlx::Error),

    #[error("migration failed: {0:?}")]
    Migrate(#[from] MigrateError),
}

impl From<Error> for Status {
    fn from(error: Error) -> Self {
        return Status::internal(format!("storage error: {0}", error));
    }
}

pub struct PgPool {
    inner: Pool<Postgres>,
}

impl PgPool {
    pub async fn connect(postgres_uri: String) -> Result<Self, Error> {
        let inner = PgPoolOptions::new()
            .max_connections(2)
            .connect(&postgres_uri)
            .await?;
        Ok(Self { inner })
    }

    pub fn inner(&self) -> &Pool<Postgres> {
        &self.inner
    }

    pub async fn run_migrations(&self) -> Result<(), Error> {
        Ok(sqlx::migrate!().run(self.inner()).await?)
    }
}
