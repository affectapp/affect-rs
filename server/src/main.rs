use affect_api::affect::{
    user_service_server::{UserService, UserServiceServer},
    *,
};
use prost_types::Timestamp;
use tonic::{transport::Server, Request, Response, Status};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Affect server");
    let addr = format!(
        "0.0.0.0:{0}",
        std::env::var("PORT").expect("env variable PORT not specified")
    )
    .parse()?;

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(affect_api::FILE_DESCRIPTOR_SET)
        .build()?;
    let user_service = UserServiceImpl::default();

    println!("Running server: {:?}", addr);
    Server::builder()
        .accept_http1(true)
        .add_service(tonic_web::enable(reflection_service))
        .add_service(tonic_web::enable(UserServiceServer::new(user_service)))
        .serve(addr)
        .await?;

    Ok(())
}

#[derive(Debug, Default)]
pub struct UserServiceImpl {}

#[tonic::async_trait]
impl UserService for UserServiceImpl {
    async fn create_user(&self, _: Request<CreateUserRequest>) -> Result<Response<User>, Status> {
        todo!()
    }

    async fn get_user(&self, _: Request<GetUserRequest>) -> Result<Response<User>, Status> {
        Ok(Response::new(User {
            user_id: Some(UserId {
                value: "test".to_string(),
            }),
            firebase_uid: "firebase_uid".to_string(),
            create_time: Some(Timestamp::default()),
            update_time: Some(Timestamp::default()),
        }))
    }

    async fn list_users(
        &self,
        _: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        todo!()
    }
}
