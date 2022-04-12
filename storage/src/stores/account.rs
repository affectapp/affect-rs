use crate::{
    models::account::*,
    sqlx::store::{PgOnDemandStore, PgTransactionalStore},
    Error,
};
use async_trait::async_trait;
use sqlx::PgExecutor;
use uuid::Uuid;

#[async_trait]
pub trait AccountStore: Sync + Send {
    /// Adds an account.
    async fn add_account(&self, new_row: NewAccountRow) -> Result<AccountRow, Error>;

    /// Lists all accounts for the provided item.
    async fn list_accounts_for_item(&self, item_id: Uuid) -> Result<Vec<AccountRow>, Error>;

    /// Deletes an account.
    async fn delete_account(&self, account_id: Uuid) -> Result<(), Error>;
}

#[async_trait]
impl AccountStore for PgOnDemandStore {
    async fn add_account(&self, new_row: NewAccountRow) -> Result<AccountRow, Error> {
        Ok(add_account(&*self.pool, new_row).await?)
    }

    async fn list_accounts_for_item(&self, item_id: Uuid) -> Result<Vec<AccountRow>, Error> {
        Ok(list_accounts_for_item(&*self.pool, item_id).await?)
    }

    async fn delete_account(&self, account_id: Uuid) -> Result<(), Error> {
        Ok(delete_account(&*self.pool, account_id).await?)
    }
}

#[async_trait]
impl<'a> AccountStore for PgTransactionalStore<'a> {
    async fn add_account(&self, new_row: NewAccountRow) -> Result<AccountRow, Error> {
        let mut lock = self.txn.lock().await;
        Ok(add_account(&mut *lock, new_row).await?)
    }

    async fn list_accounts_for_item(&self, item_id: Uuid) -> Result<Vec<AccountRow>, Error> {
        let mut lock = self.txn.lock().await;
        Ok(list_accounts_for_item(&mut *lock, item_id).await?)
    }

    async fn delete_account(&self, account_id: Uuid) -> Result<(), Error> {
        let mut lock = self.txn.lock().await;
        Ok(delete_account(&mut *lock, account_id).await?)
    }
}

async fn add_account<'a, E>(executor: E, new_row: NewAccountRow) -> Result<AccountRow, Error>
where
    E: PgExecutor<'a>,
{
    Ok(sqlx::query_file_as!(
        AccountRow,
        "queries/account/insert.sql",
        new_row.create_time,
        new_row.update_time,
        new_row.item_id,
        new_row.plaid_account_id,
        new_row.name,
        new_row.mask,
        new_row.stripe_bank_account_id,
    )
    .fetch_one(executor)
    .await?)
}

async fn list_accounts_for_item<'a, E>(executor: E, item_id: Uuid) -> Result<Vec<AccountRow>, Error>
where
    E: PgExecutor<'a>,
{
    Ok(
        sqlx::query_file_as!(AccountRow, "queries/account/list_for_item.sql", item_id)
            .fetch_all(executor)
            .await?,
    )
}

async fn delete_account<'a, E>(executor: E, account_id: Uuid) -> Result<(), Error>
where
    E: PgExecutor<'a>,
{
    sqlx::query_file!("queries/account/delete.sql", account_id)
        .execute(executor)
        .await?;
    Ok(())
}
