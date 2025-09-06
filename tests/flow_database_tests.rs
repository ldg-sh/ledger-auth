// Tests for all flows using direct database operations (without HTTP middleware)
// This validates the core business logic and database operations

use chrono::{Duration, Utc};

mod test_common;
use test_common::{TestContext, TestClient};

// ========== USER FLOW TESTS ==========

#[tokio::test]
async fn test_user_creation_database_flow() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());

    // Test admin user creation
    let (admin_id, admin_token) = client.create_test_admin().await;

    // Verify admin was created
    let admin = ctx.db.get_user_by_id(&admin_id).await.unwrap();
    assert!(admin.email.starts_with("admin-") && admin.email.ends_with("@test.com"));
    assert_eq!(admin.name, "Test Admin");
    assert!(admin.token.len() > 0);
    assert!(admin_token.len() > 0);

    // Test regular user creation
    let (user_id, user_token) = client.create_test_user().await;

    // Verify user was created
    let user = ctx.db.get_user_by_id(&user_id).await.unwrap();
    assert!(user.email.starts_with("user-") && user.email.ends_with("@test.com"));
    assert_eq!(user.name, "Test User");
    assert!(user.token.len() > 0);
    assert!(user_token.len() > 0);

    println!("✅ User creation database flow test passed!");
}

#[tokio::test]
async fn test_user_token_regeneration_database_flow() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());

    // Create user
    let (user_id, original_token) = client.create_test_user().await;
    let original_user = ctx.db.get_user_by_id(&user_id).await.unwrap();
    let original_encrypted_token = original_user.token.clone();

    // Regenerate token
    let new_raw_token = ctx.db.regenerate_user_token(&user_id).await.unwrap();

    // Verify token was changed
    let updated_user = ctx.db.get_user_by_id(&user_id).await.unwrap();
    assert_ne!(updated_user.token, original_encrypted_token);

    // Construct new full token to verify format
    use ledger_auth::utils::token::construct_token;
    let new_full_token = construct_token(&user_id, &new_raw_token);
    assert_ne!(new_full_token, original_token);
    assert!(new_full_token.len() > 0);

    println!("✅ User token regeneration database flow test passed!");
}

#[tokio::test]
async fn test_duplicate_user_email_handling() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());

    // Create first user
    let (user1_id, _user1_token) = client.create_test_user().await;

    // Get the email from the first user
    let user1 = ctx.db.get_user_by_id(&user1_id).await.unwrap();
    let user1_email = user1.email.clone();

    // Try to create another user with same email - should fail
    use ledger_auth::types::{user::DBUserCreate, token::TokenType};
    use ledger_auth::utils::token::{new_token, encrypt};

    let duplicate_token = new_token(TokenType::User);
    let encrypted_duplicate_token = encrypt(&duplicate_token).unwrap();

    let result = ctx.db.create_user(DBUserCreate {
        name: "Duplicate User".to_string(),
        email: user1_email, // Same email as the first user
        token: encrypted_duplicate_token,
    }).await;

    // Should fail due to unique constraint
    assert!(result.is_err());

    println!("✅ Duplicate email handling test passed!");
}

// ========== TEAM FLOW TESTS ==========

#[tokio::test]
async fn test_team_creation_database_flow() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());

    // Create owner
    let (owner_id, _owner_token) = client.create_test_user().await;

    // Create team
    let team_id = ctx.db.create_team(owner_id, "Test Team Database".to_string()).await.unwrap();

    // Set user as team member
    ctx.db.set_user_team(owner_id, team_id).await.unwrap();

    // Verify team was created
    let team = ctx.db.get_team(team_id).await.unwrap();
    assert_eq!(team.name, "Test Team Database");
    assert_eq!(team.owner, owner_id);

    // Verify user's team was set
    let updated_user = ctx.db.get_user_by_id(&owner_id).await.unwrap();
    assert_eq!(updated_user.team_id, Some(team_id));

    // Test team owner check
    let is_owner = ctx.db.user_is_team_owner(owner_id).await.unwrap();
    assert!(is_owner);

    println!("✅ Team creation database flow test passed!");
}

#[tokio::test]
async fn test_team_ownership_validation() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());

    // Create team with owner
    let (owner_id, _owner_token) = client.create_test_user().await;
    let team_id = client.create_team_with_owner(owner_id).await;

    // Create regular user (not owner)
    let (regular_user_id, _regular_token) = {
        use ledger_auth::types::{user::DBUserCreate, token::TokenType};
        use ledger_auth::utils::token::{new_token, encrypt, construct_token};

        let token = new_token(TokenType::User);
        let encrypted_token = encrypt(&token).unwrap();

        let user_id = ctx.db.create_user(DBUserCreate {
            name: "Regular User".to_string(),
            email: "regular@test.com".to_string(),
            token: encrypted_token,
        }).await.unwrap();

        let access_token = construct_token(&user_id, &token);
        (user_id, access_token)
    };

    // Test ownership checks
    let owner_check = ctx.db.user_is_team_owner(owner_id).await.unwrap();
    assert!(owner_check);

    let non_owner_check = ctx.db.user_is_team_owner(regular_user_id).await.unwrap();
    assert!(!non_owner_check);

    println!("✅ Team ownership validation test passed!");
}

// ========== TEAM INVITE FLOW TESTS ==========

#[tokio::test]
async fn test_team_invite_database_flow() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());

    // Create team with owner
    let (owner_id, _owner_token) = client.create_test_user().await;
    let team_id = client.create_team_with_owner(owner_id).await;

    // Create target user
    use ledger_auth::types::{user::DBUserCreate, token::TokenType};
    use ledger_auth::utils::token::{new_token, encrypt};

    let target_token = new_token(TokenType::User);
    let encrypted_target_token = encrypt(&target_token).unwrap();

    let target_user_id = ctx.db.create_user(DBUserCreate {
        name: "Invite Target".to_string(),
        email: "target@test.com".to_string(),
        token: encrypted_target_token,
    }).await.unwrap();

    // Create invite
    let expires_at = Utc::now() + Duration::minutes(30);
    let invite_id = ctx.db.create_invite(team_id, target_user_id, owner_id, expires_at).await.unwrap();

    // Verify invite was created
    let invite = ctx.db.get_invite(&invite_id).await.unwrap();
    assert_eq!(invite.team_id, team_id);
    assert_eq!(invite.user_id, target_user_id);
    assert_eq!(invite.invited_by, owner_id);
    assert!(!invite.status); // Not yet accepted
    assert!(invite.expires_at > Utc::now());

    println!("✅ Team invite database flow test passed!");
}

#[tokio::test]
async fn test_invite_acceptance_database_flow() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());

    // Setup: Create team with owner
    let (owner_id, _owner_token) = client.create_test_user().await;
    let team_id = client.create_team_with_owner(owner_id).await;

    // Create target user
    use ledger_auth::types::{user::DBUserCreate, token::TokenType};
    use ledger_auth::utils::token::{new_token, encrypt};

    let target_token = new_token(TokenType::User);
    let encrypted_target_token = encrypt(&target_token).unwrap();

    let target_user_id = ctx.db.create_user(DBUserCreate {
        name: "Invite Target".to_string(),
        email: "invite_target@test.com".to_string(),
        token: encrypted_target_token,
    }).await.unwrap();

    // Create invite
    let expires_at = Utc::now() + Duration::minutes(30);
    let invite_id = ctx.db.create_invite(team_id, target_user_id, owner_id, expires_at).await.unwrap();

    // Accept invite
    let accept_result = ctx.db.accept_invite(&invite_id).await;
    assert!(accept_result.is_ok());

    // Verify invite was accepted
    let updated_invite = ctx.db.get_invite(&invite_id).await.unwrap();
    assert!(updated_invite.status);

    // Move user to team
    ctx.db.set_user_team(target_user_id, team_id).await.unwrap();

    // Verify user was moved to team
    let updated_user = ctx.db.get_user_by_id(&target_user_id).await.unwrap();
    assert_eq!(updated_user.team_id, Some(team_id));

    // Test that the user can regenerate token (full flow simulation)
    let new_raw_token = ctx.db.regenerate_user_token(&target_user_id).await.unwrap();
    assert!(new_raw_token.len() > 0);

    println!("✅ Invite acceptance database flow test passed!");
}

#[tokio::test]
async fn test_invite_expiry_scenarios() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());

    // Setup: Create team with owner and target user
    let (owner_id, _owner_token) = client.create_test_user().await;
    let team_id = client.create_team_with_owner(owner_id).await;

    use ledger_auth::types::{user::DBUserCreate, token::TokenType};
    use ledger_auth::utils::token::{new_token, encrypt};

    let target_token = new_token(TokenType::User);
    let encrypted_target_token = encrypt(&target_token).unwrap();

    let target_user_id = ctx.db.create_user(DBUserCreate {
        name: "Expiry Test Target".to_string(),
        email: "expiry_target@test.com".to_string(),
        token: encrypted_target_token,
    }).await.unwrap();

    // Test 1: Create expired invite
    let expired_time = Utc::now() - Duration::minutes(1);
    let expired_invite_id = ctx.db.create_invite(team_id, target_user_id, owner_id, expired_time).await.unwrap();

    let expired_invite = ctx.db.get_invite(&expired_invite_id).await.unwrap();
    assert!(expired_invite.expires_at < Utc::now());

    // Test 2: Create valid invite
    let valid_time = Utc::now() + Duration::minutes(30);
    let valid_invite_id = ctx.db.create_invite(team_id, target_user_id, owner_id, valid_time).await.unwrap();

    let valid_invite = ctx.db.get_invite(&valid_invite_id).await.unwrap();
    assert!(valid_invite.expires_at > Utc::now());

    println!("✅ Invite expiry scenarios test passed!");
}

// ========== TOKEN VALIDATION TESTS ==========

#[tokio::test]
async fn test_token_validation_database_flow() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());

    // Create users with different token types
    let (admin_id, admin_token) = client.create_test_admin().await;
    let (user_id, user_token) = client.create_test_user().await;

    // Test token validation function
    use ledger_auth::utils::token::token_valid;

    // Test valid tokens
    let admin_valid = token_valid(&ctx.db, &admin_token).await;
    let user_valid = token_valid(&ctx.db, &user_token).await;

    assert!(admin_valid);
    assert!(user_valid);

    // Test invalid token
    let invalid_valid = token_valid(&ctx.db, "invalid_token_here").await;
    assert!(!invalid_valid);

    // Test token parts extraction
    use ledger_auth::utils::token::extract_token_parts;

    let admin_parts = extract_token_parts(&admin_token);
    assert!(admin_parts.is_some());
    assert_eq!(admin_parts.unwrap().0, admin_id);

    let user_parts = extract_token_parts(&user_token);
    assert!(user_parts.is_some());
    assert_eq!(user_parts.unwrap().0, user_id);

    let invalid_parts = extract_token_parts("invalid_token");
    assert!(invalid_parts.is_none());

    println!("✅ Token validation database flow test passed!");
}

// ========== COMPREHENSIVE INTEGRATION TEST ==========

#[tokio::test]
async fn test_complete_user_team_invite_flow() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());

    println!("🚀 Starting comprehensive flow test...");

    // 1. Create admin
    println!("1. Creating admin user...");
    let (admin_id, admin_token) = client.create_test_admin().await;
    assert!(admin_token.len() > 0);

    // 2. Create team owner
    println!("2. Creating team owner...");
    let (owner_id, owner_token) = client.create_test_user().await;
    assert!(owner_token.len() > 0);

    // 3. Create team
    println!("3. Creating team...");
    let team_id = client.create_team_with_owner(owner_id).await;

    // Verify team setup
    let team = ctx.db.get_team(team_id).await.unwrap();
    assert_eq!(team.owner, owner_id);

    let owner = ctx.db.get_user_by_id(&owner_id).await.unwrap();
    assert_eq!(owner.team_id, Some(team_id));
    assert!(ctx.db.user_is_team_owner(owner_id).await.unwrap());

    // 4. Create target users for invitations
    println!("4. Creating target users...");
    use ledger_auth::types::{user::DBUserCreate, token::TokenType};
    use ledger_auth::utils::token::{new_token, encrypt, construct_token};

    let mut target_users = Vec::new();
    for i in 1..=3 {
        let token = new_token(TokenType::User);
        let encrypted_token = encrypt(&token).unwrap();

        let user_id = ctx.db.create_user(DBUserCreate {
            name: format!("Target User {}", i),
            email: format!("target{}@test.com", i),
            token: encrypted_token,
        }).await.unwrap();

        let access_token = construct_token(&user_id, &token);
        target_users.push((user_id, access_token));
    }

    // 5. Create invitations
    println!("5. Creating invitations...");
    let mut invites = Vec::new();
    for (target_id, _) in &target_users {
        let invite_id = ctx.db.create_invite(
            team_id,
            *target_id,
            owner_id,
            Utc::now() + Duration::minutes(30),
        ).await.unwrap();
        invites.push(invite_id);
    }

    // 6. Accept some invitations
    println!("6. Accepting invitations...");
    for (i, invite_id) in invites.iter().enumerate() {
        if i < 2 { // Accept first 2 invitations
            ctx.db.accept_invite(invite_id).await.unwrap();
            ctx.db.set_user_team(target_users[i].0, team_id).await.unwrap();

            // Verify user was moved to team
            let updated_user = ctx.db.get_user_by_id(&target_users[i].0).await.unwrap();
            assert_eq!(updated_user.team_id, Some(team_id));
        }
    }

    // 7. Test token regeneration for team members
    println!("7. Testing token regeneration for team members...");
    for (i, (user_id, original_token)) in target_users.iter().enumerate() {
        if i < 2 { // For accepted users
            let new_raw_token = ctx.db.regenerate_user_token(user_id).await.unwrap();
            let new_full_token = construct_token(user_id, &new_raw_token);
            assert_ne!(new_full_token, *original_token);
        }
    }

    // 8. Verify final state
    println!("8. Verifying final state...");
    let final_team = ctx.db.get_team(team_id).await.unwrap();
    assert_eq!(final_team.name, "Test Team");
    assert_eq!(final_team.owner, owner_id);

    // Count accepted invites
    let mut accepted_count = 0;
    for invite_id in &invites {
        let invite = ctx.db.get_invite(invite_id).await.unwrap();
        if invite.status {
            accepted_count += 1;
        }
    }
    assert_eq!(accepted_count, 2);

    println!("✅ Complete user-team-invite flow test passed!");
    println!("   📊 Created: 1 admin, 1 owner, 3 target users, 1 team, 3 invites");
    println!("   ✅ Accepted: 2 invites, moved 2 users to team");
    println!("   🔄 Regenerated tokens for 2 team members");
}
