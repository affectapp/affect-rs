use crate::Error;
use async_trait::async_trait;

/// Store which selects connections from the pool per query.
#[async_trait]
pub trait OnDemandStore {
    // TODO: Is this trait even needed? Any functionality that should
    // be provided by all on demand stores?
}

/// Store which, while a reference exists, holds an open connection and
/// transaction. The transaction should either be committed or rolled back.
/// If neither happen, when this store is dropped the transaction will be
/// rolled back.
#[async_trait]
pub trait TransactionalStore {
    /// Commit the transaction, returning error if that fails.
    async fn commit(self) -> Result<(), Error>;

    /// Rolls back the transaction, returning error if that fails.
    async fn rollback(self) -> Result<(), Error>;
}
