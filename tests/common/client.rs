use actix_web::{web, App};
use std::sync::Arc;
use ledger_auth::{
    db::postgres_service::PostgresService,
    types::{error::AppError, token::TokenType, user::DBUserCreate},
    utils::token::{construct_token, encrypt, new_token},
};
use uuid::Uuid;

pub struct TestClient {
    pub db: Arc<PostgresService>,
}

impl TestClient {
    pub fn new(db: Arc<PostgresService>) -> Self {
        TestClient { db }
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub async fn create_test_admin(&self) -> (Uuid, String) {
        println!("[+] Creating test admin user");
        let admin_token = new_token(TokenType::Admin);
        let encrypted_token = encrypt(&admin_token).expect("Failed to encrypt token");
        let random_id = Uuid::new_v4();
        let email = format!("admin-{}@test.com", random_id);
        println!("[>] Creating admin user with email: {}", email);

        let admin_id = self.db.create_user(DBUserCreate {
            name: "Test Admin".to_string(),
            email: email.clone(),
            token: encrypted_token,
        }).await.expect("Failed to create admin");
        println!("[<] Created admin user with ID: {}", admin_id);

        let access_token = construct_token(&admin_id, &admin_token);
        println!("[+] Constructed access token for admin: {}", admin_id);

        (admin_id, access_token)
    }

    pub async fn create_test_user(&self, email: Option<String>) -> Result<(Uuid, String), AppError> {
        println!("[+] Creating test user");
        let user_token = new_token(TokenType::User);
        let encrypted_token = encrypt(&user_token).expect("Failed to encrypt token");
        let random_id = Uuid::new_v4();

        let email = email.unwrap_or_else(|| format!("user-{}@test.com", random_id));
        println!("[>] Creating user with email: {}", email);

        let user_id = self.db.create_user(DBUserCreate {
            name: "Test User".to_string(),
            email: email.clone(),
            token: encrypted_token,
        }).await?;
        println!("[<] Created user with ID: {}", user_id);

        let access_token = construct_token(&user_id, &user_token);
        println!("[+] Constructed access token for user: {}", user_id);

        Ok((user_id, access_token))
    }

    #[allow(dead_code)]
    pub async fn create_team_with_owner(&self, owner_id: Uuid) -> Uuid {
        println!("[+] Creating team for owner: {}", owner_id);
        let team_id = self.db.create_team(owner_id, "Test Team".to_string())
            .await
            .expect("Failed to create team");
        println!("[<] Created team with ID: {}", team_id);

        println!("[>] Setting user {}'s team to {}", owner_id, team_id);
        self.db.set_user_team(owner_id, team_id)
            .await
            .expect("Failed to set user team");
        println!("[<] Successfully set user's team");

        team_id
    }
}
