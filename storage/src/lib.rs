use affect_status::Status;
use async_trait::async_trait;
use futures::lock::Mutex;
use sqlx::{migrate::MigrateError, postgres::PgPoolOptions, Pool, Postgres, Transaction};
use std::{sync::Arc, time::Duration};

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

    // Returns reference to the underlying sqlx pg pool.
    pub fn inner(&self) -> &Pool<Postgres> {
        &self.inner
    }

    // Run migrations.
    pub async fn run_migrations(&self) -> Result<(), Error> {
        Ok(sqlx::migrate!().run(self.inner()).await?)
    }

    // Returns an on-demand store. This will dynamically grab connections from the
    // pool to perform sql queries/updates. This is preferred for readonly operations
    // or when a transaction is not desired.
    pub fn store(&self) -> PgOnDemandStore {
        PgOnDemandStore::new(self.inner.clone())
    }
}

// Store which selects connections from the pool per query. A transactional
// store can be created from this store using begin().
#[async_trait]
pub trait OnDemandStore<TStore>: Send + Sync
where
    TStore: TransactionalStore,
{
    // Selects a connection from the pool and starts a transaction.
    async fn begin(&self) -> Result<TStore, Error>;
}

#[derive(Debug)]
pub struct PgOnDemandStore {
    pool: Arc<Pool<Postgres>>,
}

impl PgOnDemandStore {
    fn new(pool: Arc<Pool<Postgres>>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> OnDemandStore<PgTransactionalStore<'a>> for PgOnDemandStore {
    async fn begin(&self) -> Result<PgTransactionalStore<'a>, Error> {
        let txn = self.pool.begin().await?;
        Ok(PgTransactionalStore::new(Arc::new(Mutex::new(txn))))
    }
}

// Store which, while a reference exists, holds an open connection and
// transaction. The transaction should either be committed or rolled back.
// If neither happen, when this store is dropped the transaction will be
// rolled back.
#[async_trait]
pub trait TransactionalStore: Send + Sync {
    // Commit the transaction, returning error if that fails.
    async fn commit(self) -> Result<(), Error>;

    // Rolls back the transaction, returning error if that fails.
    async fn rollback(self) -> Result<(), Error>;
}

#[derive(Debug)]
pub struct PgTransactionalStore<'a> {
    txn: Arc<Mutex<Transaction<'a, Postgres>>>,
}

impl<'a> PgTransactionalStore<'a> {
    fn new(txn: Arc<Mutex<Transaction<'a, Postgres>>>) -> Self {
        Self { txn }
    }
}

#[async_trait]
impl<'a> TransactionalStore for PgTransactionalStore<'a> {
    async fn commit(self) -> Result<(), Error> {
        let lock = Arc::try_unwrap(self.txn)
            .map_err(|e| anyhow::anyhow!("failed to unwrap transaction arc: {:?}", e))?;
        let txn = lock.into_inner();
        txn.commit().await?;
        Ok(())
    }

    async fn rollback(self) -> Result<(), Error> {
        let lock = Arc::try_unwrap(self.txn)
            .map_err(|e| anyhow::anyhow!("failed to unwrap transaction arc: {:?}", e))?;
        let txn = lock.into_inner();
        txn.rollback().await?;
        Ok(())
    }
}
