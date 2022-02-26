use affect_api::affect::{
    nonprofit_service_server::NonprofitServiceServer, user_service_server::UserServiceServer,
};
use affect_server::{
    change::{ChangeApi, ChangeCredentials},
    config::ServerConfig,
    firebase::FirebaseAuth,
    interceptors::authn::AuthnInterceptor,
    services::{nonprofit::NonprofitServiceImpl, user::UserServiceImpl},
    tonic::async_interceptor::AsyncInterceptorLayer,
};
use affect_storage::{
    stores::{nonprofit::PgNonprofitStore, user::PgUserStore},
    PgPool,
};
use log::info;
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
    env_logger::init();

    info!("Loading config");
    let config = load_config()?;

    // Database connection and stores:
    info!("Connecting to database");
    let pool = Arc::new(PgPool::connect(config.postgres.uri).await?);
    let user_store = Arc::new(PgUserStore::new(pool.clone()));
    let nonprofit_store = Arc::new(PgNonprofitStore::new(pool.clone()));

    info!("Running migrations (if any)");
    pool.run_migrations().await?;

    // Dependencies:
    let firebase_auth =
        Arc::new(FirebaseAuth::load(config.firebase.gwk_url, config.firebase.project_id).await?);
    let change_api = Arc::new(ChangeApi::new(ChangeCredentials::new(
        config.change.public_key,
        config.change.secret_key,
    )));

    // Interceptors/middleware:
    let authn_interceptor_layer = AsyncInterceptorLayer::new(AuthnInterceptor::new(
        firebase_auth.clone(),
        user_store.clone(),
    ));
    let middleware = ServiceBuilder::new()
        .timeout(Duration::from_secs(30))
        .layer(authn_interceptor_layer)
        .into_inner();

    // Services:
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(affect_api::FILE_DESCRIPTOR_SET)
        .build()?;
    let user_service = UserServiceImpl::new(user_store.clone(), firebase_auth.clone());
    let nonprofit_service = NonprofitServiceImpl::new(nonprofit_store.clone());

    let addr = format!("0.0.0.0:{0}", config.port).parse()?;
    info!("Starting server: {:?}", addr);
    Server::builder()
        .layer(middleware)
        .add_service(reflection_service)
        .add_service(UserServiceServer::new(user_service))
        .add_service(NonprofitServiceServer::new(nonprofit_service))
        .serve(addr)
        .await?;

    Ok(())
}
