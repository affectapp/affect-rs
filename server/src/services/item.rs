use crate::protobuf::into::{IntoProto, ProtoInto};
use affect_api::affect::{
    item_service_server::ItemService, CreateItemRequest, DeleteItemRequest,
    GenerateLinkTokenRequest, Item, LinkToken, ListItemsRequest, ListItemsResponse,
};
use affect_status::well_known::UnwrapField;
use affect_status::{internal, invalid_argument, not_found};
use affect_storage::database::client::DatabaseClient;
use affect_storage::database::store::{OnDemandStore, TransactionalStore};
use affect_storage::stores::item_and_account::ItemAndAccountStore;
use affect_storage::stores::user::UserStore;
use affect_storage::{
    models::account::*,
    models::item::*,
    page_token::{PageToken, PageTokenable},
    stores::{account::AccountStore, item::ItemStore},
};
use async_trait::async_trait;
use chrono::Utc;
use prost_types::Timestamp;
use std::marker::PhantomData;
use std::{
    cmp::{max, min},
    sync::Arc,
};
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct ItemServiceImpl<Db, Store, TStore> {
    database: Arc<Db>,
    plaid: Arc<plaid::Client>,
    stripe: Arc<stripe::Client>,
    _marker: PhantomData<(Store, TStore)>,
}

impl<Db, Store, TStore> ItemServiceImpl<Db, Store, TStore> {
    pub fn new(database: Arc<Db>, plaid: Arc<plaid::Client>, stripe: Arc<stripe::Client>) -> Self {
        Self {
            database,
            plaid,
            stripe,
            _marker: PhantomData,
        }
    }
}

#[async_trait]
impl<Db, Store, TStore> ItemService for ItemServiceImpl<Db, Store, TStore>
where
    Db: DatabaseClient<Store, TStore> + 'static,
    Store: ItemStore + AccountStore + UserStore + OnDemandStore + 'static,
    TStore: ItemStore + AccountStore + TransactionalStore + 'static,
{
    async fn generate_link_token(
        &self,
        request: Request<GenerateLinkTokenRequest>,
    ) -> Result<Response<LinkToken>, Status> {
        let message = request.into_inner();
        let user_id = message
            .user_id
            .unwrap_field("user_id")?
            .proto_field_into("user_id")?;
        let user_row = self
            .database
            .on_demand()
            .find_user_by_id(user_id)
            .await?
            .ok_or(not_found!("user not found"))?;

        #[allow(deprecated)]
        let plaid_response = self
            .plaid
            .create_link_token(&plaid::CreateLinkTokenRequest {
                client_name: "Affect".to_string(),
                language: plaid::SupportedLanguage::en,
                country_codes: [plaid::SupportedCountry::US].to_vec(),
                user: plaid::EndUser {
                    client_user_id: user_row.user_id.to_string(),
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
            .unwrap_field("user_id")?
            .proto_field_into("user_id")?;
        let user_row = self
            .database
            .on_demand()
            .find_user_by_id(user_id)
            .await?
            .ok_or(not_found!("user not found"))?;

        let plaid_item_response = self
            .plaid
            .exchange_public_token(&message.plaid_public_token)
            .await
            .map_err(|e| invalid_argument!("failed to exchange public token: {:?}", e))?;

        let plaid_accounts_response = self
            .plaid
            .accounts(&plaid_item_response.access_token)
            .await
            .map_err(|e| internal!("failed to fetch accounts: {:?}", e))?;
        let plaid_account = match plaid_accounts_response.accounts.into_iter().nth(0) {
            Some(account) => account,
            None => {
                return Err(internal!("expected only 1 linked account"));
            }
        };

        let plaid_stripe_response = self
            .plaid
            .stripe_create_bank_account_token(
                &plaid_item_response.access_token,
                &plaid_account.account_id,
            )
            .await
            .map_err(|e| internal!("failed to create stripe bank account token: {:?}", e))?;

        let stripe_payment_source_params =
            stripe::PaymentSourceParams::Token(stripe::TokenId::Bank(
                plaid_stripe_response
                    .stripe_bank_account_token
                    .parse()
                    .map_err(|e| internal!("failed to parse stripe bank account token: {:?}", e))?,
            ));
        let stripe_payment_source = match stripe::Customer::attach_source(
            &self.stripe,
            &user_row
                .stripe_customer_id
                .parse()
                .map_err(|e| internal!("failed to parse stripe customer id: {:?}", e))?,
            stripe_payment_source_params,
        )
        .await
        .map_err(|e| internal!("failed to attach source to stripe customer: {:?}", e))?
        {
            stripe::PaymentSource::BankAccount(bank_account) => bank_account,
            _ => {
                return Err(internal!(
                    "expected bank account payment source returned from stripe"
                ))
            }
        };

        let now = Utc::now();
        let item_row = self
            .database
            .on_demand()
            .add_item(NewItemRow {
                create_time: now,
                update_time: now,
                user_id,
                plaid_item_id: plaid_item_response.item_id,
                plaid_access_token: plaid_item_response.access_token,
            })
            .await?;

        let account_rows = vec![
            self.database
                .on_demand()
                .add_account(NewAccountRow {
                    create_time: now,
                    update_time: now,
                    item_id: item_row.item_id.clone(),
                    plaid_account_id: plaid_account.account_id,
                    name: plaid_account.name,
                    mask: plaid_account.mask,
                    stripe_bank_account_id: stripe_payment_source.id.to_string(),
                })
                .await?,
        ];
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
        let user_id = message
            .user_id
            .unwrap_field("user_id")?
            .proto_field_into("user_id")?;

        let (rows_plus_one, total_count) = self
            .database
            .on_demand()
            .list_and_count_items_for_user((page_size + 1).into(), page_token, user_id)
            .await?;

        let (page_rows, next_page_rows) =
            rows_plus_one.split_at(min(rows_plus_one.len(), page_size as usize));

        // Map rows to protos and serialize page token.
        let mut items = Vec::new();
        for row in page_rows {
            let account_rows = self
                .database
                .on_demand()
                .list_accounts_for_item(row.item_id)
                .await?;
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

    async fn delete_item(
        &self,
        request: Request<DeleteItemRequest>,
    ) -> Result<Response<()>, Status> {
        let message = request.into_inner();
        let item_id = message
            .item_id
            .unwrap_field("item_id")?
            .proto_field_into("item_id")?;

        let store = self.database.begin().await?;
        let item = store
            .find_item_by_id(item_id)
            .await?
            .ok_or(not_found!("item not found"))?;
        let accounts = store.list_accounts_for_item(item.item_id).await?;
        let account_ids = accounts
            .iter()
            .map(|account| account.account_id.clone())
            .collect::<Vec<Uuid>>();
        store
            .delete_item_and_accounts(item.item_id, account_ids)
            .await?;
        store.commit().await?;

        // Detach bank accounts from stripe customers.
        let user = self
            .database
            .on_demand()
            .find_user_by_id(item.user_id)
            .await?
            .ok_or(internal!("user not found"))?;
        let customer_id = user
            .stripe_customer_id
            .parse()
            .map_err(|e| internal!("failed to parse stripe customer id: {:?}", e))?;
        for account in accounts {
            let source_id = stripe::PaymentSourceId::BankAccount(
                account
                    .stripe_bank_account_id
                    .parse()
                    .map_err(|e| internal!("failed to parse stripe bank account id: {:?}", e))?,
            );
            stripe::Customer::detach_source(&self.stripe, &customer_id, &source_id)
                .await
                .map_err(|e| internal!("failed to detach source from stripe customer: {:?}", e))?;
        }

        Ok(Response::new(()))
    }
}
