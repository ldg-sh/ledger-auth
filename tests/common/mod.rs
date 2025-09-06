use std::sync::Arc;
use testcontainers::{Container, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;
use ledger_auth::db::postgres_service::PostgresService;
use ledger_auth::config::EnvConfig;

pub mod client;

pub struct TestContext {
    pub db: Arc<PostgresService>,
    pub _container: Container<Postgres>,
}

impl TestContext {
    pub async fn new() -> TestContext {
        let postgres = Postgres::default();
        let container = postgres.start().await.expect("Failed to start postgres container");

        let host = container.get_host().await.expect("Failed to get host");
        let port = container.get_host_port_ipv4(5432).await.expect("Failed to get port");

        let db_url = format!("postgresql://postgres:postgres@{}:{}/postgres", host, port);

        let db = Arc::new(
            PostgresService::new(&db_url)
                .await
                .expect("Failed to initialize PostgresService")
        );

        TestContext {
            db,
            _container: container,
        }
    }
}

pub fn get_test_config() -> EnvConfig {
    EnvConfig {
        port: 8080,
        db_url: "test".to_string(), // Not used in tests
        grpc: ledger_auth::config::GrpcConfig { port: 50051 },
        mail: ledger_auth::config::MailConfig {
            api_key: "test".to_string(),
            endpoint: "test".to_string(),
        },
    }
}

// Test data helpers
pub mod test_data {
    use ledger_auth::types::user::RUserCreate;
    use ledger_auth::types::team::RTeamCreate;
    use uuid::Uuid;

    pub fn sample_user() -> RUserCreate {
        RUserCreate {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        }
    }

    pub fn sample_user_with_email(email: &str) -> RUserCreate {
        RUserCreate {
            name: "Test User".to_string(),
            email: email.to_string(),
        }
    }

    pub fn sample_team(owner_id: Uuid) -> RTeamCreate {
        RTeamCreate {
            name: "Test Team".to_string(),
            owner: owner_id,
        }
    }
}
