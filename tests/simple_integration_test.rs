use testcontainers::{runners::AsyncRunner, ContainerAsync};
use testcontainers_modules::postgres::Postgres;
use std::sync::Arc;
use ledger_auth::{
    db::postgres_service::PostgresService,
    types::{user::DBUserCreate, token::TokenType},
    utils::token::{new_token, encrypt, construct_token}
};
use uuid::Uuid;

struct TestSetup {
    pub db: Arc<PostgresService>,
    _container: ContainerAsync<Postgres>,
}

impl TestSetup {
    pub async fn new() -> Self {
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

        TestSetup {
            db,
            _container: container,
        }
    }

    pub async fn create_test_user(&self) -> (Uuid, String) {
        let user_token = new_token(TokenType::User);
        let encrypted_token = encrypt(&user_token).expect("Failed to encrypt token");
        
        let user_id = self.db.create_user(DBUserCreate {
            name: "Test User".to_string(),
            email: "user@test.com".to_string(),
            token: encrypted_token,
        }).await.expect("Failed to create user");

        let access_token = construct_token(&user_id, &user_token);
        
        (user_id, access_token)
    }
}

#[tokio::test]
async fn test_database_setup_and_user_creation() {
    let setup = TestSetup::new().await;
    
    // Test basic user creation in database
    let (user_id, _token) = setup.create_test_user().await;
    
    // Verify user was created
    let user = setup.db.get_user_by_id(&user_id).await;
    assert!(user.is_ok());
    
    let user = user.unwrap();
    assert_eq!(user.email, "user@test.com");
    assert_eq!(user.name, "Test User");
}

#[tokio::test]
async fn test_team_operations() {
    let setup = TestSetup::new().await;
    
    // Create user
    let (user_id, _token) = setup.create_test_user().await;
    
    // Create team
    let team_id = setup.db.create_team(user_id, "Test Team".to_string()).await;
    assert!(team_id.is_ok());
    
    let team_id = team_id.unwrap();
    
    // Set user as team member
    let result = setup.db.set_user_team(user_id, team_id).await;
    assert!(result.is_ok());
    
    // Verify team was created
    let team = setup.db.get_team(team_id).await;
    assert!(team.is_ok());
    
    let team = team.unwrap();
    assert_eq!(team.name, "Test Team");
    assert_eq!(team.owner, user_id);
    
    // Verify user's team was set
    let updated_user = setup.db.get_user_by_id(&user_id).await.unwrap();
    assert_eq!(updated_user.team_id, Some(team_id));
}

#[tokio::test]
async fn test_invite_operations() {
    let setup = TestSetup::new().await;
    
    // Create team owner
    let (owner_id, _owner_token) = setup.create_test_user().await;
    let team_id = setup.db.create_team(owner_id, "Test Team".to_string()).await.unwrap();
    setup.db.set_user_team(owner_id, team_id).await.unwrap();
    
    // Create target user with different email
    let target_token = new_token(TokenType::User);
    let encrypted_target_token = encrypt(&target_token).expect("Failed to encrypt token");
    
    let target_id = setup.db.create_user(DBUserCreate {
        name: "Target User".to_string(),
        email: "target@test.com".to_string(),
        token: encrypted_target_token,
    }).await.expect("Failed to create target user");
    
    // Create invite
    use chrono::{Duration, Utc};
    let invite_result = setup.db.create_invite(
        team_id,
        target_id,
        owner_id,
        Utc::now() + Duration::minutes(30),
    ).await;
    
    assert!(invite_result.is_ok());
    let invite_id = invite_result.unwrap();
    
    // Get invite and verify
    let invite = setup.db.get_invite(&invite_id).await;
    assert!(invite.is_ok());
    
    let invite = invite.unwrap();
    assert_eq!(invite.team_id, team_id);
    assert_eq!(invite.user_id, target_id);
    assert_eq!(invite.invited_by, owner_id);
    assert!(!invite.status);
    
    // Accept invite
    let accept_result = setup.db.accept_invite(&invite_id).await;
    assert!(accept_result.is_ok());
    
    // Verify invite was accepted
    let updated_invite = setup.db.get_invite(&invite_id).await.unwrap();
    assert!(updated_invite.status);
}

#[tokio::test]
async fn test_token_operations() {
    let setup = TestSetup::new().await;
    
    // Create user
    let (user_id, original_token) = setup.create_test_user().await;
    
    // Test token validation (would need to implement token_valid function directly)
    // For now, just test token regeneration
    
    let new_token_result = setup.db.regenerate_user_token(&user_id).await;
    assert!(new_token_result.is_ok());
    
    let new_token = new_token_result.unwrap();
    assert_ne!(new_token, original_token); // Should be different
    
    // Verify user's token was updated
    let updated_user = setup.db.get_user_by_id(&user_id).await.unwrap();
    // Note: We can't easily verify the encrypted token matches without exposing internals
    assert!(updated_user.token.len() > 0);
}