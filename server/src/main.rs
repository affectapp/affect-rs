use affect_api::affect::{
    affiliate_service_server::AffiliateServiceServer, cause_service_server::CauseServiceServer,
    item_service_server::ItemServiceServer, nonprofit_service_server::NonprofitServiceServer,
    user_service_server::UserServiceServer,
};
use affect_server::{
    change::client::{ChangeClient, ChangeCredentials},
    config::ServerConfig,
    firebase::FirebaseAuth,
    interceptors::authn::AuthnInterceptor,
    seed,
    services::{
        affiliate::AffiliateServiceImpl, cause::CauseServiceImpl, item::ItemServiceImpl,
        nonprofit::NonprofitServiceImpl, user::UserServiceImpl,
    },
    tonic::async_interceptor::AsyncInterceptorLayer,
};
use affect_storage::{database::client::DatabaseClient, sqlx::client::PgDatabaseClient};
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

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    info!("Loading config");
    let config = load_config()?;

    // Database connection and stores:
    info!("Connecting to database");
    let database = Arc::new(PgDatabaseClient::connect(config.postgres.uri).await?);
    let store = Arc::new(database.on_demand());

    info!("Running migrations (if any)");
    database.run_migrations().await?;

    // Dependencies:
    let firebase_auth =
        Arc::new(FirebaseAuth::load(config.firebase.gwk_url, config.firebase.project_id).await?);
    let change_client = Arc::new(ChangeClient::new(ChangeCredentials::new(
        config.change.public_key,
        config.change.secret_key,
    )));
    let plaid_client = Arc::new(plaid::Client::new(
        config.plaid.client_id,
        config.plaid.secret_key,
        config.plaid.env.parse()?,
    ));
    let stripe_client = Arc::new(stripe::Client::new(config.stripe.secret));

    // Seed database with data.
    seed::insert_nonprofits(store.clone(), change_client).await?;

    // Interceptors/middleware:
    let authn_interceptor_layer =
        AsyncInterceptorLayer::new(AuthnInterceptor::new(firebase_auth.clone(), store.clone()));
    let middleware = ServiceBuilder::new()
        .timeout(Duration::from_secs(30))
        .layer(authn_interceptor_layer)
        .into_inner();

    // Services:
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(affect_api::FILE_DESCRIPTOR_SET)
        .build()?;
    let user_service =
        UserServiceImpl::new(store.clone(), firebase_auth.clone(), stripe_client.clone());
    let nonprofit_service = NonprofitServiceImpl::new(database.clone());
    let item_service = ItemServiceImpl::new(
        database.clone(),
        plaid_client.clone(),
        stripe_client.clone(),
    );
    let cause_service = CauseServiceImpl::new(database.clone());
    let affiliate_service = AffiliateServiceImpl::new(database.clone(), stripe_client.clone());

    let port: u16 = match (config.port, config.port_env_var) {
        (None, Some(port_env_var)) => std::env::var(&port_env_var)?.parse()?,
        (Some(port), None) => port,
        _ => panic!("Expected"),
    };
    let addr = format!("0.0.0.0:{0}", port).parse()?;
    info!("Starting server: {:?}", addr);
    Server::builder()
        .layer(middleware)
        .add_service(reflection_service)
        .add_service(UserServiceServer::new(user_service))
        .add_service(NonprofitServiceServer::new(nonprofit_service))
        .add_service(ItemServiceServer::new(item_service))
        .add_service(CauseServiceServer::new(cause_service))
        .add_service(AffiliateServiceServer::new(affiliate_service))
        .serve(addr)
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = run().await;
    if result.is_err() {
        panic!("Failed to run: {:?}", result);
    }
    Ok(())
}
