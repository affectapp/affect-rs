use affect_api::affect::{nonprofit_service_server::NonprofitService, ListNonprofitsRequest, *};
use affect_storage::{
    page_token::PageToken,
    stores::nonprofit::{NonprofitPageToken, NonprofitRow, NonprofitStore},
};
use async_trait::async_trait;
use prost_types::Timestamp;
use std::{
    cmp::{max, min},
    sync::Arc,
};
use tonic::{Request, Response, Status};

pub struct NonprofitServiceImpl {
    nonprofit_store: Arc<dyn NonprofitStore>,
}

impl NonprofitServiceImpl {
    pub fn new(nonprofit_store: Arc<dyn NonprofitStore>) -> Self {
        Self { nonprofit_store }
    }
}

fn nonprofit_row_to_proto(row: NonprofitRow) -> Nonprofit {
    Nonprofit {
        nonprofit_id: Some(NonprofitId {
            value: row.nonprofit_id.to_string(),
        }),
        create_time: Some(Timestamp {
            seconds: row.create_time.timestamp(),
            nanos: row.create_time.timestamp_subsec_nanos() as i32,
        }),
        update_time: Some(Timestamp {
            seconds: row.update_time.timestamp(),
            nanos: row.update_time.timestamp_subsec_nanos() as i32,
        }),
        change_nonprofit_id: row.change_nonprofit_id,
        icon_url: row.icon_url,
        title: row.title,
        ein: row.ein,
        mission: row.mission,
        category: row.category,
    }
}

#[async_trait]
impl NonprofitService for NonprofitServiceImpl {
    async fn list_nonprofits(
        &self,
        request: Request<ListNonprofitsRequest>,
    ) -> Result<Response<ListNonprofitsResponse>, Status> {
        let page_size = min(max(request.get_ref().page_size, 10), 100);
        let page_token =
            NonprofitPageToken::deserialize_from_page_token(&request.get_ref().page_token)
                .or(Err(Status::invalid_argument("'page_token' is invalid")))?;

        let (rows_plus_one, total_count) = self
            .nonprofit_store
            .list_and_count_nonprofits((page_size + 1).into(), page_token)
            .await?;

        let (rows, extra_rows) =
            rows_plus_one.split_at(min(rows_plus_one.len(), page_size as usize));

        // Map rows to protos and serialize page token.
        let nonprofits = rows
            .iter()
            .map(|row| nonprofit_row_to_proto(row.clone()))
            .collect();
        let next_page_token = extra_rows
            .first()
            .map(|next_row| {
                let next_page_token: NonprofitPageToken = next_row.clone().into();
                next_page_token.serialize_as_page_token()
            })
            .unwrap_or(Ok("".to_string()))?;

        Ok(Response::new(ListNonprofitsResponse {
            nonprofits,
            next_page_token,
            total_count,
        }))
    }
}
