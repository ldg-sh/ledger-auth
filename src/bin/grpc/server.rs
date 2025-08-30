use tonic::transport::Server;
use ledger_auth::grpc::{authentication, pb};
use ledger_auth::config::config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    println!("Starting gRPC server..");
    let address = format!("0.0.0.0:{}", config().grpc.port);

    println!("Server started on port {}", config().grpc.port);
    Server::builder()
        .add_service(authentication::server())
        .serve(address.parse()?)
        .await?;

    Ok(())
}
