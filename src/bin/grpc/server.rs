use tonic::transport::Server;
use ledger_auth::grpc::{authentication, pb};
use ledger_auth::config;
use std::sync::Arc;
use ledger_auth::db::postgres_service::PostgresService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let config = config::EnvConfig::from_env();
    config::CONFIG.set(config.clone()).unwrap();
    println!("Starting gRPC server..");
    let address = format!("0.0.0.0:{}", config.grpc.port);

    println!("Starting postgres...");
    let postgres_service = Arc::new(
        PostgresService::new(
            &config.db_url,
        )
            .await
            .expect("Failed to initialize PostgresService")
    );
    println!("Started postgres!");

    println!("Server started on port {}", config.grpc.port);
    Server::builder()
        .add_service(authentication::server(postgres_service.clone()))
        .serve(address.parse()?)
        .await?;

    Ok(())
}
