use actix_web::{test, web, App};
use std::sync::Arc;
use ledger_auth::{
    db::postgres_service::PostgresService,
    routes::configure_routes,
    types::{
        token::TokenType,
    },
    utils::token::{new_token, encrypt, construct_token},
};
use uuid::Uuid;

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
            .configure(configure_routes)
    }

    pub async fn create_test_admin(&self) -> (Uuid, String) {
        let admin_token = new_token(TokenType::Admin);
        let encrypted_token = encrypt(&admin_token).expect("Failed to encrypt token");

        let admin_id = self.db.create_user(ledger_auth::types::user::DBUserCreate {
            name: "Test Admin".to_string(),
            email: "admin@test.com".to_string(),
            token: encrypted_token,
        }).await.expect("Failed to create admin");

        let access_token = construct_token(&admin_id, &admin_token);

        (admin_id, access_token)
    }

    pub async fn create_test_user(&self) -> (Uuid, String) {
        let user_token = new_token(TokenType::User);
        let encrypted_token = encrypt(&user_token).expect("Failed to encrypt token");

        let user_id = self.db.create_user(ledger_auth::types::user::DBUserCreate {
            name: "Test User".to_string(),
            email: "user@test.com".to_string(),
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
