use crate::{
    database::store::TransactionalStore,
    stores::{account::AccountStore, item::ItemStore},
    Error,
};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait ItemAndAccountStore {
    /// Deletes an item and accounts.
    async fn delete_item_and_accounts(
        &self,
        item_id: Uuid,
        account_ids: Vec<Uuid>,
    ) -> Result<(), Error>;
}

/// Implementation of the store for items and accounts, operations that should be combined into
/// a transaction.
#[async_trait]
impl<S> ItemAndAccountStore for S
where
    S: ItemStore + AccountStore + TransactionalStore,
{
    async fn delete_item_and_accounts(
        &self,
        item_id: Uuid,
        account_ids: Vec<Uuid>,
    ) -> Result<(), Error> {
        for account_id in account_ids {
            self.delete_account(account_id).await?;
        }
        self.delete_item(item_id).await?;
        Ok(())
    }
}
