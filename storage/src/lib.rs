use affect_status::Status;
use futures::{lock::Mutex, Future};
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
    inner: Pool<Postgres>,
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

    pub fn on_demand<'a>(&'a self) -> PgOnDemandStore<'a> {
        PgOnDemandStore::new(Arc::new(&self.inner))
    }

    pub async fn transactional<'a>(&self) -> Result<PgTransactionalStore<'a>, Error> {
        let txn = self.inner().begin().await?;
        Ok(PgTransactionalStore::new(Arc::new(Mutex::new(txn))))
    }

    pub async fn with_transaction<'a, R, Fut>(
        &self,
        fun: impl FnOnce(PgTransactionalStore<'a>) -> Fut,
    ) -> Result<R, Error>
    where
        Fut: Future<Output = R>,
    {
        let txn = self.transactional().await?;
        Ok(fun(txn).await)
    }
}

pub struct PgOnDemandStore<'a> {
    pool: Arc<&'a Pool<Postgres>>,
}

impl<'a> PgOnDemandStore<'a> {
    pub fn new(pool: Arc<&'a Pool<Postgres>>) -> Self {
        Self { pool }
    }
}

pub struct PgTransactionalStore<'a> {
    txn: Arc<Mutex<Transaction<'a, Postgres>>>,
}

impl<'a> PgTransactionalStore<'a> {
    pub fn new(txn: Arc<Mutex<Transaction<'a, Postgres>>>) -> Self {
        Self { txn }
    }

    // Commit the transaction, returning error if that fails.
    // Consumes self.
    pub async fn commit(self) -> Result<(), Error> {
        let lock = Arc::try_unwrap(self.txn).expect("lock still has multiple owners");
        let txn = lock.into_inner();
        txn.commit().await?;
        Ok(())
    }

    // Rolls back the transaction, returning error if that fails.
    // Consumes self.
    pub async fn rollback(self) -> Result<(), Error> {
        let lock = Arc::try_unwrap(self.txn).expect("lock still has multiple owners");
        let txn = lock.into_inner();
        txn.rollback().await?;
        Ok(())
    }
}
