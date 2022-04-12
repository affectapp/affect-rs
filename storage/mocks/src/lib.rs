use affect_storage::{
    database::{
        client::DatabaseClient,
        store::{OnDemandStore, TransactionalStore},
    },
    models::cause::*,
    stores::cause::*,
    Error,
};
use async_trait::async_trait;
use mockall::mock;
use uuid::Uuid;

mock! {
  pub DatabaseClient {}

  #[async_trait]
  impl DatabaseClient<MockStore, MockStore> for DatabaseClient {
      fn on_demand(&self) -> MockStore;

      async fn begin(&self) -> Result<MockStore, Error>;
  }
}

mock! {
  pub Store {}

  #[async_trait]
  impl CauseStore for Store {
      async fn add_cause(&self, new_row: NewCauseRow) -> Result<CauseRow, Error>;

      async fn list_causes_for_user(
          &self,
          page_size: i64,
          page_token: Option<CausePageToken>,
          user_id: Uuid,
      ) -> Result<Vec<FullCauseRow>, Error>;

      async fn count_causes_for_user(&self, user_id: Uuid) -> Result<i64, Error>;

      async fn add_cause_recipient(
          &self,
          new_row: NewCauseRecipientRow,
      ) -> Result<CauseRecipientRow, Error>;

      async fn list_cause_recipients_for_cause(
          &self,
          cause_id: Uuid,
      ) -> Result<Vec<CauseRecipientRow>, Error>;
  }

  #[async_trait]
  impl OnDemandStore for Store {
  }

  #[async_trait]
  impl TransactionalStore for Store {
      async fn commit(self) -> Result<(), Error>;

      async fn rollback(self) -> Result<(), Error>;
  }
}
