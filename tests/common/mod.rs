use testcontainers::{runners::AsyncRunner, ContainerAsync};
use testcontainers_modules::postgres::Postgres;
use std::sync::Arc;
use ledger_auth::{
    db::postgres_service::PostgresService,
    config::EnvConfig,
};

pub mod client;

pub struct TestContext {
    pub db: Arc<PostgresService>,
    pub _container: ContainerAsync<Postgres>,
}

impl TestContext {
    pub async fn new() -> TestContext {
        // Initialize config for tests
        let test_config = get_test_config();
        let _ = ledger_auth::config::CONFIG.set(test_config);

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
        admin_key: "test_admin_key".to_string(),
        resend_key: "test_resend_key".to_string(),
        grpc: ledger_auth::config::GrpcConfig {
            port: 50051,
            auth_key: "test_grpc_auth".to_string(),
        },
    }
}


pub mod test_data {
    use ledger_auth::types::user::RUserCreate;
    use ledger_auth::types::team::RTeamCreate;
    use uuid::Uuid;

    #[allow(dead_code)]
    pub fn sample_user() -> RUserCreate {
        RUserCreate {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn sample_user_with_email(email: &str) -> RUserCreate {
        RUserCreate {
            name: "Test User".to_string(),
            email: email.to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn sample_team(owner_id: Uuid) -> RTeamCreate {
        RTeamCreate {
            name: "Test Team".to_string(),
            owner: owner_id,
        }
    }
}
