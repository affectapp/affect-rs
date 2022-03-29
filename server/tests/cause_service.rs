use affect_api::affect::{
    cause_service_client::CauseServiceClient, cause_service_server::CauseServiceServer,
    CauseRecipient, CreateCauseRequest,
};
use affect_server::services::cause::CauseServiceImpl;
use affect_storage::{
    database::{
        client::DatabaseClient,
        store::{OnDemandStore, TransactionalStore},
    },
    stores::cause::*,
};
use async_trait::async_trait;
use chrono::Utc;
use mockall::{mock, Sequence};
use std::sync::Arc;
use std::time::Duration;
use tonic::{transport::Server, Request};
use uuid::Uuid;

#[tokio::test]
async fn create_cause() -> Result<(), anyhow::Error> {
    let mut database = MockDatabaseClient::new();
    let mut txn = MockStore::new();

    // Transaction
    {
        let mut seq = Sequence::new();
        txn.expect_add_cause()
            .times(1)
            .in_sequence(&mut seq)
            .return_once(|_| {
                Ok(CauseRow {
                    cause_id: Uuid::new_v4(),
                    create_time: Utc::now(),
                    update_time: Utc::now(),
                    user_id: Uuid::new_v4(),
                    name: "some name".to_string(),
                })
            });
        txn.expect_add_cause_recipient()
            .times(1)
            .in_sequence(&mut seq)
            .return_once(|_| {
                Ok(CauseRecipientRow {
                    cause_id: Uuid::new_v4(),
                    nonprofit_id: Uuid::new_v4(),
                    create_time: Utc::now(),
                    update_time: Utc::now(),
                })
            });
        txn.expect_commit()
            .times(1)
            .in_sequence(&mut seq)
            .return_once(|| Ok(()));
    }

    // Begin transaction.
    database.expect_begin().times(1).return_once(|| Ok(txn));

    let cause_service = CauseServiceServer::new(CauseServiceImpl::new(Arc::new(database)));
    let addr = "127.0.0.1:54321";
    let server = tokio::spawn(async move {
        Server::builder()
            .add_service(cause_service)
            .serve(addr.parse().unwrap())
            .await
            .expect("failed to start test server");
    });
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut channel = CauseServiceClient::connect(format!("http://{0}", addr)).await?;
    channel
        .create_cause(Request::new(CreateCauseRequest {
            user_id: Uuid::new_v4().to_string(),
            recipients: [CauseRecipient {
                cause_id: "unused".to_string(),
                nonprofit_id: Uuid::new_v4().to_string(),
            }]
            .to_vec(),
        }))
        .await?;

    server.abort();
    Ok(())
}

mock! {
    pub DatabaseClient {}

    #[async_trait]
    impl DatabaseClient<MockStore, MockStore> for DatabaseClient {
        fn on_demand(&self) -> MockStore;

        async fn begin(&self) -> Result<MockStore, affect_storage::Error>;
    }
}

mock! {
    pub Store {}

    #[async_trait]
    impl CauseStore for Store {
        async fn add_cause(&self, new_row: NewCauseRow) -> Result<CauseRow, affect_storage::Error>;

        async fn list_causes_for_user(
            &self,
            page_size: i64,
            page_token: Option<CausePageToken>,
            user_id: Uuid,
        ) -> Result<Vec<CauseRow>, affect_storage::Error>;

        async fn count_causes_for_user(&self, user_id: Uuid) -> Result<i64, affect_storage::Error>;

        async fn add_cause_recipient(
            &self,
            new_row: NewCauseRecipientRow,
        ) -> Result<CauseRecipientRow, affect_storage::Error>;

        async fn list_cause_recipients_for_cause(
            &self,
            cause_id: Uuid,
        ) -> Result<Vec<CauseRecipientRow>, affect_storage::Error>;
    }

    #[async_trait]
    impl OnDemandStore for Store {
    }

    #[async_trait]
    impl TransactionalStore for Store {
        async fn commit(self) -> Result<(), affect_storage::Error>;

        async fn rollback(self) -> Result<(), affect_storage::Error>;
    }
}
