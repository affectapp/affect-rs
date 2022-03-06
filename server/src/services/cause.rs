use affect_api::affect::{
    cause_service_server::CauseService, Cause, CauseRecipient, CreateCauseRequest,
    ListCausesRequest, ListCausesResponse,
};
use affect_storage::{
    page_token::{PageToken, PageTokenable},
    stores::{
        cause::{CausePageToken, CauseRow, CauseStore, NewCauseRow},
        cause_recipient::{CauseRecipientRow, CauseRecipientStore, NewCauseRecipientRow},
    },
    PgPool, PgTransactionalStore,
};
use async_trait::async_trait;
use chrono::Utc;
use prost_types::Timestamp;
use std::{
    cmp::{max, min},
    sync::Arc,
};
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct CauseServiceImpl {
    pool: Arc<PgPool>,
}

impl CauseServiceImpl {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
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
impl CauseService for CauseServiceImpl {
    async fn list_causes(
        &self,
        request: Request<ListCausesRequest>,
    ) -> Result<Response<ListCausesResponse>, Status> {
        let message = request.into_inner();

        let page_size = min(max(message.page_size, 1), 100);
        let page_token = CausePageToken::deserialize_page_token(&message.page_token)
            .map_err(|e| Status::invalid_argument(format!("'page_token' is invalid: {:?}", e)))?;
        let user_id = Some(message.user_id)
            .filter(|s| !s.is_empty())
            .ok_or(Status::invalid_argument("'user_id' must be specified"))?
            .parse::<Uuid>()
            .map_err(|e| Status::invalid_argument(format!("'user_id' is invalid: {:?}", e)))?;

        let (rows_plus_one, total_count) = self
            .pool
            .on_demand()
            .list_and_count_causes_for_user((page_size + 1).into(), page_token, user_id)
            .await?;

        let (page_rows, next_page_rows) =
            rows_plus_one.split_at(min(rows_plus_one.len(), page_size as usize));

        // Map rows to protos and serialize page token.
        let mut causes = Vec::new();
        for row in page_rows {
            let cause_recipient_rows = self
                .pool
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

    async fn create_cause(
        &self,
        request: Request<CreateCauseRequest>,
    ) -> Result<Response<Cause>, Status> {
        let message = request.into_inner();
        let (cause_row, cause_recipient_rows) = self
            .pool
            .with_transaction(
                |store| async move { insert_cause_and_recipients(&store, &message).await },
            )
            .await??;
        Ok(Response::new(row_to_proto(cause_row, cause_recipient_rows)))
    }
}

async fn insert_cause_and_recipients<'a>(
    store: &PgTransactionalStore<'a>,
    message: &CreateCauseRequest,
) -> Result<(CauseRow, Vec<CauseRecipientRow>), Status> {
    let user_id = message
        .user_id
        .parse::<Uuid>()
        .map_err(|e| Status::invalid_argument(format!("'user_id' is invalid: {:?}", e)))?;

    let now = Utc::now();
    let cause_row = store
        .add_cause(NewCauseRow {
            create_time: now,
            update_time: now,
            user_id,
            name: "some name".to_string(),
        })
        .await?;

    let mut recipient_rows = Vec::new();
    for recipient in &message.recipients {
        let nonprofit_id = recipient
            .nonprofit_id
            .parse::<Uuid>()
            .map_err(|e| Status::invalid_argument(format!("'nonprofit_id' is invalid: {:?}", e)))?;

        recipient_rows.push(
            store
                .add_cause_recipient(NewCauseRecipientRow {
                    cause_id: cause_row.cause_id.clone(),
                    nonprofit_id,
                    create_time: now,
                    update_time: now,
                })
                .await?,
        );
    }
    Ok((cause_row, recipient_rows))
}
