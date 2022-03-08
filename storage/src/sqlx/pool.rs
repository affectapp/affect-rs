use crate::{sqlx::store::PgOnDemandStore, Error};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{sync::Arc, time::Duration};

// Wrapper around sqlx::Pool<Postgres>.
pub struct PgPool {
    inner: Arc<Pool<Postgres>>,
}

impl PgPool {
    // Connects to the provided postgres URI and returns the connected pool.
    pub async fn connect(postgres_uri: String) -> Result<Self, Error> {
        // let connect_options = PgConnectOptions::new();
        let inner = PgPoolOptions::new()
            .max_connections(1)
            .connect_timeout(Duration::from_secs(5))
            .connect(&postgres_uri)
            .await?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    // Provides access to the underlying sqlx pool.
    pub fn inner(&self) -> &Pool<Postgres> {
        &self.inner
    }

    // Run migrations.
    pub async fn run_migrations(&self) -> Result<(), Error> {
        Ok(sqlx::migrate!().run(&*self.inner).await?)
    }

    // Returns an on-demand store. This will dynamically grab connections from the
    // pool to perform sql queries/updates. This is preferred for readonly operations
    // or when a transaction is not desired.
    pub fn store(&self) -> PgOnDemandStore {
        PgOnDemandStore::new(self.inner.clone())
    }
}
