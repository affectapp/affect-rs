use affect_api::affect::{
    item_service_server::ItemService, CreateItemRequest, GenerateLinkTokenRequest, Item, LinkToken,
    ListItemsRequest, ListItemsResponse,
};
use affect_storage::{
    page_token::{PageToken, PageTokenable},
    stores::item::{ItemPageToken, ItemRow, ItemStore},
};
use async_trait::async_trait;
use prost_types::Timestamp;
use std::{
    cmp::{max, min},
    sync::Arc,
};
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct ItemServiceImpl {
    item_store: Arc<dyn ItemStore>,
}

impl ItemServiceImpl {
    pub fn new(item_store: Arc<dyn ItemStore>) -> Self {
        Self { item_store }
    }
}

fn item_row_to_proto(row: ItemRow) -> Item {
    Item {
        item_id: row.item_id.to_string(),
        create_time: Some(Timestamp {
            seconds: row.create_time.timestamp(),
            nanos: row.create_time.timestamp_subsec_nanos() as i32,
        }),
        update_time: Some(Timestamp {
            seconds: row.update_time.timestamp(),
            nanos: row.update_time.timestamp_subsec_nanos() as i32,
        }),
        user_id: row.user_id.to_string(),
        accounts: [].to_vec(),
    }
}

#[async_trait]
impl ItemService for ItemServiceImpl {
    async fn list_items(
        &self,
        request: Request<ListItemsRequest>,
    ) -> Result<Response<ListItemsResponse>, Status> {
        let message = request.into_inner();

        let page_size = min(max(message.page_size, 1), 100);
        let page_token = ItemPageToken::deserialize_page_token(&message.page_token)
            .map_err(|e| Status::invalid_argument(format!("'page_token' is invalid: {:?}", e)))?;
        let user_id = Some(message.user_id)
            .filter(|s| !s.is_empty())
            .ok_or(Status::invalid_argument("'user_id' must be specified"))?
            .parse::<Uuid>()
            .map_err(|e| Status::invalid_argument(format!("'user_id' is invalid: {:?}", e)))?;

        let (rows_plus_one, total_count) = self
            .item_store
            .list_and_count_items_for_user((page_size + 1).into(), page_token, user_id)
            .await?;

        let (page_rows, next_page_rows) =
            rows_plus_one.split_at(min(rows_plus_one.len(), page_size as usize));

        // Map rows to protos and serialize page token.
        let items = page_rows
            .iter()
            .map(|row| item_row_to_proto(row.clone()))
            .collect();

        // Next page token or empty string.
        let next_page_token = next_page_rows
            .first()
            .map(|next_row| next_row.page_token().serialize_page_token())
            .unwrap_or(Ok("".to_string()))?;

        Ok(Response::new(ListItemsResponse {
            items,
            next_page_token,
            total_count,
        }))
    }

    async fn generate_link_token(
        &self,
        _request: Request<GenerateLinkTokenRequest>,
    ) -> Result<Response<LinkToken>, Status> {
        todo!()
    }

    async fn create_item(
        &self,
        _request: Request<CreateItemRequest>,
    ) -> Result<Response<Item>, Status> {
        todo!()
    }
}
