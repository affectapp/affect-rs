use affect_api::affect::{
    cause_service_server::CauseService, Cause, CauseRecipient, CreateCauseRequest,
    ListCausesRequest, ListCausesResponse,
};
use affect_status::invalid_argument;
use affect_storage::{
    database::{
        client::DatabaseClient,
        store::{OnDemandStore, TransactionalStore},
    },
    page_token::{PageToken, PageTokenable},
    stores::{
        cause::{CausePageToken, CauseRow, CauseStore},
        cause_and_recipient::CauseAndRecipientStore,
        cause_recipient::{CauseRecipientRow, CauseRecipientStore},
    },
};
use async_trait::async_trait;
use prost_types::Timestamp;
use std::{
    cmp::{max, min},
    marker::PhantomData,
    sync::Arc,
};
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct CauseServiceImpl<Client, Store, TStore>
where
    Client: DatabaseClient<Store, TStore>,
    Store: CauseStore + CauseRecipientStore + OnDemandStore,
    TStore: CauseStore + CauseRecipientStore + TransactionalStore,
{
    database: Arc<Client>,
    _marker: PhantomData<fn() -> (Store, TStore)>,
}

impl<Client, Store, TStore> CauseServiceImpl<Client, Store, TStore>
where
    Client: DatabaseClient<Store, TStore>,
    Store: CauseStore + CauseRecipientStore + OnDemandStore,
    TStore: CauseStore + CauseRecipientStore + TransactionalStore,
{
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            database: client,
            _marker: PhantomData,
        }
    }
}

fn row_to_proto(row: CauseRow, cause_recipient_rows: Vec<CauseRecipientRow>) -> Cause {
    Cause {
        cause_id: row.cause_id.to_string(),
        create_time: Some(Timestamp {
            seconds: row.create_time.timestamp(),
            nanos: row.create_time.timestamp_subsec_nanos() as i32,
        }),
        update_time: Some(Timestamp {
            seconds: row.update_time.timestamp(),
            nanos: row.update_time.timestamp_subsec_nanos() as i32,
        }),
        user_id: row.user_id.to_string(),
        recipients: cause_recipient_rows
            .into_iter()
            .map(|cause_recipient_row| CauseRecipient {
                cause_id: cause_recipient_row.cause_id.to_string(),
                nonprofit_id: cause_recipient_row.nonprofit_id.to_string(),
            })
            .collect(),
    }
}

#[async_trait]
impl<Client, Store, TStore> CauseService for CauseServiceImpl<Client, Store, TStore>
where
    Client: DatabaseClient<Store, TStore> + 'static,
    Store: CauseStore + CauseRecipientStore + OnDemandStore + 'static,
    TStore: CauseStore + CauseRecipientStore + TransactionalStore + 'static,
{
    async fn create_cause(
        &self,
        request: Request<CreateCauseRequest>,
    ) -> Result<Response<Cause>, Status> {
        let message = request.into_inner();

        let user_id = message
            .user_id
            .parse::<Uuid>()
            .map_err(|e| invalid_argument!("'user_id' is invalid: {:?}", e))?;

        let mut recipient_nonprofit_ids = Vec::new();
        for recipient in message.recipients {
            recipient_nonprofit_ids.push(
                recipient
                    .nonprofit_id
                    .parse::<Uuid>()
                    .map_err(|e| invalid_argument!("'nonprofit_id' is invalid: {:?}", e))?,
            );
        }

        let txn = self.database.begin().await?;
        let (cause_row, cause_recipient_rows) = txn
            .add_cause_and_recipients(user_id, recipient_nonprofit_ids)
            .await?;
        txn.commit().await?;

        Ok(Response::new(row_to_proto(cause_row, cause_recipient_rows)))
    }

    async fn list_causes(
        &self,
        request: Request<ListCausesRequest>,
    ) -> Result<Response<ListCausesResponse>, Status> {
        let message = request.into_inner();

        let page_size = min(max(message.page_size, 1), 100);
        let page_token = CausePageToken::deserialize_page_token(&message.page_token)
            .map_err(|e| invalid_argument!("'page_token' is invalid: {:?}", e))?;
        let user_id = Some(message.user_id)
            .filter(|s| !s.is_empty())
            .ok_or(Status::invalid_argument("'user_id' must be specified"))?
            .parse::<Uuid>()
            .map_err(|e| invalid_argument!("'user_id' is invalid: {:?}", e))?;

        let (rows_plus_one, total_count) = self
            .database
            .on_demand()
            .list_and_count_causes_for_user((page_size + 1).into(), page_token, user_id)
            .await?;

        let (page_rows, next_page_rows) =
            rows_plus_one.split_at(min(rows_plus_one.len(), page_size as usize));

        // Map rows to protos and serialize page token.
        let mut causes = Vec::new();
        for row in page_rows {
            let cause_recipient_rows = self
                .database
                .on_demand()
                .list_cause_recipients_for_cause(row.cause_id)
                .await?;
            causes.push(row_to_proto(row.clone(), cause_recipient_rows));
        }

        // Next page token or empty string.
        let next_page_token = next_page_rows
            .first()
            .map(|next_row| next_row.page_token().serialize_page_token())
            .unwrap_or(Ok("".to_string()))?;

        Ok(Response::new(ListCausesResponse {
            causes,
            next_page_token,
            total_count,
        }))
    }
}
