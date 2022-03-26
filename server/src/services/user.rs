use crate::{firebase::FirebaseAuth, protobuf::into::IntoProto};
use affect_api::affect::{get_user_request::Identifier, user_service_server::UserService, *};
use affect_status::{internal, invalid_argument, not_found};
use affect_storage::stores::user::{NewUserRow, UserStore};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
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

        let decoded_id_token = self
            .firebase_auth
            .verify_id_token(message.firebase_id_token)
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
            Some(Identifier::FirebaseUserId(firebase_user_id)) => Ok(self
                .user_store
                .find_user_by_firebase_uid(firebase_user_id)
                .await?),
            Some(Identifier::UserId(_)) => todo!(),
            None => Err(invalid_argument!("must specify identifier")),
        }?
        .ok_or(not_found!("user not found"))?;

        Ok(Response::new(user_row.into_proto()?))
    }

    async fn list_users(
        &self,
        _: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        todo!()
    }
}
