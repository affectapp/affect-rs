use crate::firebase::FirebaseAuth;
use affect_api::affect::{get_user_request::Identifier, user_service_server::UserService, *};
use affect_storage::stores::user::{NewUserRow, UserRow, UserStore};
use chrono::Utc;
use prost_types::Timestamp;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct UserServiceImpl {
    user_store: Arc<dyn UserStore>,
    firebase_auth: Arc<FirebaseAuth>,
}

impl UserServiceImpl {
    pub fn new(user_store: Arc<dyn UserStore>, firebase_auth: Arc<FirebaseAuth>) -> Self {
        Self {
            user_store,
            firebase_auth,
        }
    }
}

fn user_row_to_proto(user_row: UserRow) -> User {
    User {
        user_id: user_row.user_id.to_string(),
        create_time: Some(Timestamp {
            seconds: user_row.create_time.timestamp(),
            nanos: user_row.create_time.timestamp_subsec_nanos() as i32,
        }),
        update_time: Some(Timestamp {
            seconds: user_row.update_time.timestamp(),
            nanos: user_row.update_time.timestamp_subsec_nanos() as i32,
        }),
        firebase_uid: user_row.firebase_uid,
    }
}

#[tonic::async_trait]
impl UserService for UserServiceImpl {
    async fn create_user(&self, req: Request<CreateUserRequest>) -> Result<Response<User>, Status> {
        let message = req.into_inner();

        let decoded_id_token = self
            .firebase_auth
            .verify_id_token(message.firebase_id_token)
            .map_err(|_| Status::invalid_argument("firebase id token verification failed"))?;
        let now = Utc::now();

        let user_row = self
            .user_store
            .add_user(NewUserRow {
                create_time: now,
                update_time: now,
                firebase_uid: decoded_id_token.uid,
                firebase_email: decoded_id_token.email,
            })
            .await?;

        Ok(Response::new(user_row_to_proto(user_row)))
    }

    async fn get_user(&self, req: Request<GetUserRequest>) -> Result<Response<User>, Status> {
        let message = req.into_inner();
        let user_row = match message.identifier {
            Some(Identifier::FirebaseUserId(firebase_user_id)) => Ok(self
                .user_store
                .find_user_by_firebase_uid(firebase_user_id)
                .await?),
            Some(Identifier::UserId(_)) => todo!(),
            None => Err(Status::invalid_argument("must specify identifier")),
        }?
        .ok_or(Status::not_found("user not found"))?;

        Ok(Response::new(user_row_to_proto(user_row)))
    }

    async fn list_users(
        &self,
        _: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        todo!()
    }
}
