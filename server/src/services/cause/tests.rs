use crate::services::cause::CauseServiceImpl;
use affect_api::affect::{
    cause_service_client::CauseServiceClient, cause_service_server::CauseServiceServer,
    CauseRecipient, CreateCauseRequest,
};
use affect_storage::models::cause::*;
use affect_storage_mocks::*;
use chrono::Utc;
use mockall::Sequence;
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
