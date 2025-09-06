use testcontainers::{runners::AsyncRunner, ContainerAsync};
use testcontainers_modules::postgres::Postgres;
use std::sync::Arc;
use ledger_auth::{
    db::postgres_service::PostgresService,
    config::EnvConfig,
    types::{user::DBUserCreate, token::TokenType},
    utils::token::{new_token, encrypt, construct_token},
};
use uuid::Uuid;
use actix_web::{web, App};

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

pub struct TestClient {
    pub db: Arc<PostgresService>,
}

impl TestClient {
    pub fn new(db: Arc<PostgresService>) -> Self {
        TestClient { db }
    }

    pub fn create_app(&self) -> actix_web::App<
        impl actix_web::dev::ServiceFactory<
            actix_web::dev::ServiceRequest,
            Config = (),
            Response = actix_web::dev::ServiceResponse,
            Error = actix_web::Error,
            InitError = (),
        >,
    > {
        App::new()
            .app_data(web::Data::new(Arc::clone(&self.db)))
            .configure(ledger_auth::routes::configure_routes)
    }

    pub async fn create_test_admin(&self) -> (Uuid, String) {
        let admin_token = new_token(TokenType::Admin);
        let encrypted_token = encrypt(&admin_token).expect("Failed to encrypt token");
        let random_id = Uuid::new_v4();
        
        let admin_id = self.db.create_user(DBUserCreate {
            name: "Test Admin".to_string(),
            email: format!("admin-{}@test.com", random_id),
            token: encrypted_token,
        }).await.expect("Failed to create admin");

        let access_token = construct_token(&admin_id, &admin_token);
        
        (admin_id, access_token)
    }

    pub async fn create_test_user(&self) -> (Uuid, String) {
        let user_token = new_token(TokenType::User);
        let encrypted_token = encrypt(&user_token).expect("Failed to encrypt token");
        let random_id = Uuid::new_v4();
        
        let user_id = self.db.create_user(DBUserCreate {
            name: "Test User".to_string(),
            email: format!("user-{}@test.com", random_id),
            token: encrypted_token,
        }).await.expect("Failed to create user");

        let access_token = construct_token(&user_id, &user_token);
        
        (user_id, access_token)
    }

    pub async fn create_team_with_owner(&self, owner_id: Uuid) -> Uuid {
        let team_id = self.db.create_team(owner_id, "Test Team".to_string())
            .await
            .expect("Failed to create team");
        
        self.db.set_user_team(owner_id, team_id)
            .await
            .expect("Failed to set user team");
            
        team_id
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