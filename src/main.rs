use crate::db::postgres_service::PostgresService;
use crate::routes::configure_routes;
use actix_web::{web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use std::sync::Arc;
use utils::webutils::validate_token;

mod config;
mod db;
mod routes;
mod response;
mod types;
mod utils;
mod macros;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let config = config::EnvConfig::from_env();
    config::CONFIG.set(config.clone()).unwrap();
    
    let addr = format!("0.0.0.0:{}", config.port);

    let postgres_service = Arc::new(
        PostgresService::new(
            &config.db_url,
        )
            .await
            .expect("Failed to initialize PostgresService")
    );



    println!("Starting server on {}", addr);


    HttpServer::new(move || {
        let auth = HttpAuthentication::bearer(validate_token);

        App::new()
            .wrap(auth)
            .app_data(web::Data::new(Arc::clone(&postgres_service)))
            .configure(configure_routes)
    })
    .bind(addr)?
    .run()
    .await
}
