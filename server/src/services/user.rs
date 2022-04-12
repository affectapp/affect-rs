use crate::{
    firebase::FirebaseAuth,
    protobuf::into::{IntoProto, ProtoInto},
};
use affect_api::affect::{get_user_request::Identifier, user_service_server::UserService, *};
use affect_status::{internal, invalid_argument, not_found, well_known::UnwrapField};
use affect_storage::{
    models::user::*,
    page_token::{PageToken, PageTokenable},
    stores::user::UserStore,
};
use async_trait::async_trait;
use chrono::Utc;
use std::{
    cmp::{max, min},
    sync::Arc,
};
use tonic::{Request, Response, Status};

pub struct UserServiceImpl {
    user_store: Arc<dyn UserStore>,
    firebase_auth: Arc<FirebaseAuth>,
    stripe_client: Arc<stripe::Client>,
}

impl UserServiceImpl {
    pub fn new(
        user_store: Arc<dyn UserStore>,
        firebase_auth: Arc<FirebaseAuth>,
        stripe_client: Arc<stripe::Client>,
    ) -> Self {
        Self {
            user_store,
            firebase_auth,
            stripe_client,
        }
    }
}

#[async_trait]
impl UserService for UserServiceImpl {
    async fn create_user(&self, req: Request<CreateUserRequest>) -> Result<Response<User>, Status> {
        let message = req.into_inner();

        let firebase_id_token = message
            .firebase_id_token
            .unwrap_field("firebase_id_token")?;

        let decoded_id_token = self
            .firebase_auth
            .verify_id_token(firebase_id_token)
            .map_err(|e| invalid_argument!("firebase id token verification failed: {:?}", e))?;
        let now = Utc::now();

        let email = decoded_id_token.email;

        let mut create_customer = stripe::CreateCustomer::new();
        create_customer.email = Some(&email);

        let stripe_customer = stripe::Customer::create(&self.stripe_client, create_customer)
            .await
            .map_err(|e| internal!("failed to fetch accounts: {:?}", e))?;

        let user_row = self
            .user_store
            .add_user(NewUserRow {
                create_time: now,
                update_time: now,
                firebase_uid: decoded_id_token.uid,
                firebase_email: email,
                stripe_customer_id: stripe_customer.id.to_string(),
            })
            .await?;

        Ok(Response::new(user_row.into_proto()?))
    }

    async fn get_user(&self, req: Request<GetUserRequest>) -> Result<Response<User>, Status> {
        let message = req.into_inner();
        let user_row = match message.identifier {
            Some(Identifier::FirebaseUserId(firebase_user_id)) => {
                self.user_store
                    .find_user_by_firebase_uid(firebase_user_id)
                    .await?
            }
            Some(Identifier::UserId(user_id)) => {
                self.user_store
                    .find_user_by_id(user_id.proto_field_into("user_id")?)
                    .await?
            }
            None => return Err(invalid_argument!("must specify identifier")),
        }
        .ok_or(not_found!("user not found"))?;

        Ok(Response::new(user_row.into_proto()?))
    }

    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        let message = request.into_inner();

        let page_size = min(max(message.page_size, 1), 100);
        let limit: i64 = (page_size + 1).into();
        let page_token = UserPageToken::deserialize_page_token(&message.page_token)
            .map_err(|e| invalid_argument!("'page_token' is invalid: {:?}", e))?;

        let (rows_plus_one, total_count) = self
            .user_store
            .list_and_count_users(limit, page_token)
            .await?;

        let (page_rows, next_page_rows) =
            rows_plus_one.split_at(min(rows_plus_one.len(), page_size as usize));

        // Map rows to protos and serialize page token.
        let mut users: Vec<User> = Vec::new();
        for row in page_rows {
            users.push(row.clone().into_proto()?);
        }

        // Next page token or empty string.
        let next_page_token = next_page_rows
            .first()
            .map(|next_row| next_row.page_token().serialize_page_token())
            .unwrap_or(Ok("".to_string()))?;

        Ok(Response::new(ListUsersResponse {
            users,
            next_page_token,
            total_count,
        }))
    }
}
