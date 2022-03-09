use crate::{
    database::client::DatabaseClient,
    sqlx::store::{PgOnDemandStore, PgTransactionalStore},
    Error,
};
use async_trait::async_trait;
use futures::lock::Mutex;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{sync::Arc, time::Duration};

/// Wrapper around sqlx::Pool<Postgres>.
pub struct PgDatabaseClient {
    inner: Arc<Pool<Postgres>>,
}

impl PgDatabaseClient {
    /// Connects to the provided postgres URI and returns the connected client.
    pub async fn connect(postgres_uri: String) -> Result<Self, Error> {
        let inner = PgPoolOptions::new()
            .max_connections(1)
            .connect_timeout(Duration::from_secs(5))
            .connect(&postgres_uri)
            .await?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    /// Provides access to the underlying sqlx pool.
    pub fn inner(&self) -> &Pool<Postgres> {
        &self.inner
    }

    /// Run migrations.
    pub async fn run_migrations(&self) -> Result<(), Error> {
        Ok(sqlx::migrate!().run(&*self.inner).await?)
    }
}

#[async_trait]
impl<'a> DatabaseClient<PgOnDemandStore, PgTransactionalStore<'a>> for PgDatabaseClient {
    /// Returns an on-demand store. This will dynamically grab connections from the
    /// pool to perform sql queries/updates. This is preferred for readonly operations
    /// or when a transaction is not desired.
    fn on_demand(&self) -> PgOnDemandStore {
        PgOnDemandStore::new(self.inner.clone())
    }

    /// Opens a connection to the database and starts a transaction on the connection.
    async fn begin(&self) -> Result<PgTransactionalStore<'a>, Error> {
        let txn = self.inner.begin().await?;
        Ok(PgTransactionalStore::new(Arc::new(Mutex::new(txn))))
    }
}
