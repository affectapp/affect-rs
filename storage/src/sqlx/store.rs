use crate::Error;
use async_trait::async_trait;
use futures::lock::Mutex;
use sqlx::{Pool, Postgres, Transaction};
use std::sync::Arc;

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
    pub(crate) pool: Arc<Pool<Postgres>>,
}

impl PgOnDemandStore {
    pub(crate) fn new(pool: Arc<Pool<Postgres>>) -> Self {
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
    pub(crate) txn: Arc<Mutex<Transaction<'a, Postgres>>>,
}

impl<'a> PgTransactionalStore<'a> {
    pub(crate) fn new(txn: Arc<Mutex<Transaction<'a, Postgres>>>) -> Self {
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
