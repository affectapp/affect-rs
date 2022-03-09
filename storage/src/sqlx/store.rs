use crate::{
    database::store::{OnDemandStore, TransactionalStore},
    Error,
};
use async_trait::async_trait;
use futures::lock::Mutex;
use sqlx::{Pool, Postgres, Transaction};
use std::sync::Arc;

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
impl OnDemandStore for PgOnDemandStore {}

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
