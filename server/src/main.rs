use affect_api::affect::user_service_server::UserServiceServer;
use affect_server::{
    async_interceptor::AsyncInterceptorLayer, config::ServerConfig, firebase::FirebaseAuth,
    interceptors::authn::AuthnInterceptor, services::user::UserServiceImpl,
};
use affect_storage::user::PostgresUserStore;
use std::{sync::Arc, time::Duration};
use tonic::transport::Server;
use tower::ServiceBuilder;

fn load_config() -> Result<ServerConfig, Box<dyn std::error::Error>> {
    let config_path = std::env::var("CONFIG_PATH").ok();
    let config = std::env::var("CONFIG").ok();

    let config_str = match (config_path, config) {
        (None, Some(config)) => config,
        (Some(config_path), None) => std::fs::read_to_string(config_path)?,
        (Some(_), Some(_)) => {
            panic!("Only one of CONFIG and CONFIG_PATH environment variables should be specified")
        }
        (None, None) => {
            panic!("Either CONFIG or CONFIG_PATH environment variables should be specified")
        }
    };

    Ok(toml::from_str::<ServerConfig>(&config_str)?)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Loading config");
    let config = load_config()?;

    let port_str = std::env::var("PORT")
        .ok()
        .or_else(|| {
            println!("PORT environment variable unspecified, using default port");
            Some("50051".to_string())
        })
        .unwrap();
    let addr = format!("0.0.0.0:{0}", port_str).parse()?;

    let user_store = Arc::new(PostgresUserStore::connect(config.postgres.uri).await?);
    let firebase_auth =
        Arc::new(FirebaseAuth::load(config.firebase.gwk_url, config.firebase.project_id).await?);

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(affect_api::FILE_DESCRIPTOR_SET)
        .build()?;
    let user_service = UserServiceImpl::new(user_store.clone(), firebase_auth.clone());

    let authn_layer: AsyncInterceptorLayer<AuthnInterceptor> = AsyncInterceptorLayer::new(
        AuthnInterceptor::new(firebase_auth.clone(), user_store.clone()),
    );

    let middleware = ServiceBuilder::new()
        .timeout(Duration::from_secs(30))
        .layer(authn_layer)
        .into_inner();

    println!("Starting server: {:?}", addr);
    Server::builder()
        .accept_http1(true)
        .layer(middleware)
        .add_service(tonic_web::enable(reflection_service))
        .add_service(tonic_web::enable(UserServiceServer::new(user_service)))
        .serve(addr)
        .await?;

    Ok(())
}
