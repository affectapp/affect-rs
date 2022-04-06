use crate::protobuf::into::{IntoProto, ProtoInto};
use affect_api::affect::{
    cause_service_server::CauseService, Cause, CreateCauseRequest, ListCausesRequest,
    ListCausesResponse,
};
use affect_status::{invalid_argument, well_known::UnwrapField};
use affect_storage::{
    database::{
        client::DatabaseClient,
        store::{OnDemandStore, TransactionalStore},
    },
    page_token::{PageToken, PageTokenable},
    stores::cause::{CauseAndRecipientStore, CausePageToken, CauseStore},
};
use async_trait::async_trait;
use std::{
    cmp::{max, min},
    marker::PhantomData,
    sync::Arc,
};
use tonic::{Request, Response, Status};

pub struct CauseServiceImpl<Db, Store, TStore> {
    database: Arc<Db>,
    _marker: PhantomData<fn() -> (Store, TStore)>,
}

impl<Db, Store, TStore> CauseServiceImpl<Db, Store, TStore> {
    pub fn new(client: Arc<Db>) -> Self {
        Self {
            database: client,
            _marker: PhantomData,
        }
    }
}

#[async_trait]
impl<Db, Store, TStore> CauseService for CauseServiceImpl<Db, Store, TStore>
where
    Db: DatabaseClient<Store, TStore> + 'static,
    Store: CauseStore + OnDemandStore + 'static,
    TStore: CauseStore + TransactionalStore + 'static,
{
    async fn create_cause(
        &self,
        request: Request<CreateCauseRequest>,
    ) -> Result<Response<Cause>, Status> {
        let message = request.into_inner();

        let user_id = message
            .user_id
            .unwrap_field("user_id")?
            .proto_field_into("user_id")?;

        let mut recipient_nonprofit_ids = Vec::new();
        for recipient in message.recipients {
            recipient_nonprofit_ids.push(
                recipient
                    .nonprofit_id
                    .unwrap_field("nonprofit_id")?
                    .proto_field_into("nonprofit_id")?,
            );
        }

        let txn = self.database.begin().await?;
        let (cause_row, cause_recipient_rows) = txn
            .add_cause_and_recipients(user_id, recipient_nonprofit_ids)
            .await?;
        txn.commit().await?;

        Ok(Response::new(
            (cause_row, cause_recipient_rows).into_proto()?,
        ))
    }

    async fn list_causes(
        &self,
        request: Request<ListCausesRequest>,
    ) -> Result<Response<ListCausesResponse>, Status> {
        let message = request.into_inner();

        let page_size = min(max(message.page_size, 1), 100);
        let page_token = CausePageToken::deserialize_page_token(&message.page_token)
            .map_err(|e| invalid_argument!("'page_token' is invalid: {:?}", e))?;
        let user_id = message
            .user_id
            .unwrap_field("user_id")?
            .proto_field_into("user_id")?;

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
            causes.push(row.clone().into_proto()?);
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
