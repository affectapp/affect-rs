use crate::prost::into::IntoProto;
use affect_api::affect::{
    item_service_server::ItemService, CreateItemRequest, GenerateLinkTokenRequest, Item, LinkToken,
    ListItemsRequest, ListItemsResponse,
};
use affect_status::{internal, invalid_argument};
use affect_storage::{
    page_token::{PageToken, PageTokenable},
    stores::{
        account::{AccountStore, NewAccountRow},
        item::{ItemPageToken, ItemStore, NewItemRow},
    },
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

pub struct ItemServiceImpl<Store>
where
    Store: ItemStore + AccountStore,
{
    store: Arc<Store>,
    plaid: Arc<plaid::Client>,
}

impl<Store> ItemServiceImpl<Store>
where
    Store: ItemStore + AccountStore,
{
    pub fn new(store: Arc<Store>, plaid: Arc<plaid::Client>) -> Self {
        Self { store, plaid }
    }
}

#[async_trait]
impl<Store> ItemService for ItemServiceImpl<Store>
where
    Store: ItemStore + AccountStore + 'static,
{
    async fn generate_link_token(
        &self,
        request: Request<GenerateLinkTokenRequest>,
    ) -> Result<Response<LinkToken>, Status> {
        let message = request.into_inner();
        // TODO: Validate message.user_id.

        #[allow(deprecated)]
        let plaid_response = self
            .plaid
            .create_link_token(&plaid::CreateLinkTokenRequest {
                client_name: "Affect".to_string(),
                language: plaid::SupportedLanguage::en,
                country_codes: [plaid::SupportedCountry::US].to_vec(),
                user: plaid::EndUser {
                    client_user_id: message.user_id,
                },
                products: [plaid::SupportedProduct::Transactions].to_vec(),
                webhook: None,
                access_token: None,
                link_customization_name: None,
                redirect_uri: None,
                android_package_name: None,
                account_filters: None,
                institution_id: None,
                payment_initiation: None,
            })
            .await
            .map_err(|e| internal!("failed to generate plaid link token: {:?}", e))?;

        Ok(Response::new(LinkToken {
            plaid_link_token: plaid_response.link_token,
            expire_time: Some(Timestamp {
                seconds: plaid_response.expiration.timestamp(),
                nanos: plaid_response.expiration.timestamp_subsec_nanos() as i32,
            }),
        }))
    }

    async fn create_item(
        &self,
        request: Request<CreateItemRequest>,
    ) -> Result<Response<Item>, Status> {
        let message = request.into_inner();
        let user_id = message
            .user_id
            .parse()
            .map_err(|e| invalid_argument!("'user_id' is invalid: {:?}", e))?;
        let plaid_response = self
            .plaid
            .exchange_public_token(&message.plaid_public_token)
            .await
            .map_err(|e| invalid_argument!("failed to exchange public token: {:?}", e))?;

        let now = Utc::now();
        let item_row = self
            .store
            .add_item(NewItemRow {
                create_time: now,
                update_time: now,
                user_id,
                plaid_item_id: plaid_response.item_id,
                plaid_access_token: plaid_response.access_token,
            })
            .await?;

        let plaid_account_response = self
            .plaid
            .accounts(&item_row.plaid_access_token)
            .await
            .map_err(|e| internal!("failed to fetch accounts: {:?}", e))?;

        let mut account_rows = Vec::new();
        for plaid_account in plaid_account_response.accounts {
            let now = Utc::now();
            account_rows.push(
                self.store
                    .add_account(NewAccountRow {
                        create_time: now,
                        update_time: now,
                        item_id: item_row.item_id.clone(),
                        plaid_account_id: plaid_account.account_id,
                        name: plaid_account.name,
                        mask: plaid_account.mask,
                    })
                    .await?,
            );
        }
        Ok(Response::new((item_row, account_rows).into_proto()?))
    }

    async fn list_items(
        &self,
        request: Request<ListItemsRequest>,
    ) -> Result<Response<ListItemsResponse>, Status> {
        let message = request.into_inner();

        let page_size = min(max(message.page_size, 1), 100);
        let page_token = ItemPageToken::deserialize_page_token(&message.page_token)
            .map_err(|e| invalid_argument!("'page_token' is invalid: {:?}", e))?;
        let user_id = Some(message.user_id)
            .filter(|s| !s.is_empty())
            .ok_or(Status::invalid_argument("'user_id' must be specified"))?
            .parse::<Uuid>()
            .map_err(|e| invalid_argument!("'user_id' is invalid: {:?}", e))?;

        let (rows_plus_one, total_count) = self
            .store
            .list_and_count_items_for_user((page_size + 1).into(), page_token, user_id)
            .await?;

        let (page_rows, next_page_rows) =
            rows_plus_one.split_at(min(rows_plus_one.len(), page_size as usize));

        // Map rows to protos and serialize page token.
        let mut items = Vec::new();
        for row in page_rows {
            let account_rows = self.store.list_accounts_for_item(row.item_id).await?;
            items.push((row.clone(), account_rows).into_proto()?);
        }

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
}
