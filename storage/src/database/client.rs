use crate::{
    database::store::{OnDemandStore, TransactionalStore},
    Error,
};
use async_trait::async_trait;

/// Provides access to a database.
#[async_trait]
pub trait DatabaseClient<S, T>
where
    Self: Send + Sync,
    S: OnDemandStore,
    T: TransactionalStore,
{
    /// Returns an on-demand store. This will dynamically grab connections from the
    /// pool to perform sql queries/updates. This is preferred for readonly operations
    /// or when a transaction is not desired.
    fn on_demand(&self) -> S;

    /// Returns a transactional store. This will open a connection and start a
    /// transaction. You _should_ call `commit()` or `rollback()` on the resulting
    /// store, but the transaction will be rolled back otherwise upon the store
    /// being dropped.
    async fn begin(&self) -> Result<T, Error>;
}
