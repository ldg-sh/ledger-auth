#[path = "common/mod.rs"]
mod common;

use actix_web::{test, http::StatusCode};
use common::{TestContext, test_data, client::TestClient};
use ledger_auth::types::team::RTeamInviteUser;

#[tokio::test]
async fn test_team_creation_flow_success() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());
    let app = test::init_service(client.create_app()).await;

    // Create admin user for authentication
    let (_admin_id, admin_token) = client.create_test_admin().await;

    // Create a user who will own the team
    let (owner_id, _owner_token) = client.create_test_user().await;

    let team_data = test_data::sample_team(owner_id);

    let req = test::TestRequest::post()
        .uri("/team/create")
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(&team_data)
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["message"].as_str().unwrap().contains("successfully created"));
    assert!(body["id"].is_string());

    let team_id_str = body["id"].as_str().unwrap();
    let team_id = uuid::Uuid::parse_str(team_id_str).unwrap();

    // Verify team was created in database
    let created_team = ctx.db.get_team(team_id).await;
    assert!(created_team.is_ok());

    let team = created_team.unwrap();
    assert_eq!(team.name, team_data.name);
    assert_eq!(team.owner, owner_id);

    // Verify user's team_id was set
    let updated_user = ctx.db.get_user_by_id(&owner_id).await.unwrap();
    assert_eq!(updated_user.team_id, Some(team_id));
}

#[tokio::test]
async fn test_team_creation_flow_unauthorized() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());
    let app = test::init_service(client.create_app()).await;

    let (owner_id, _) = client.create_test_user().await;
    let team_data = test_data::sample_team(owner_id);

    let req = test::TestRequest::post()
        .uri("/team/create")
        .insert_header(("Authorization", "Bearer invalid_token"))
        .set_json(&team_data)
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_team_creation_flow_user_token_forbidden() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());
    let app = test::init_service(client.create_app()).await;

    let (owner_id, user_token) = client.create_test_user().await;
    let team_data = test_data::sample_team(owner_id);

    let req = test::TestRequest::post()
        .uri("/team/create")
        .insert_header(("Authorization", format!("Bearer {}", user_token)))
        .set_json(&team_data)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Should be forbidden since regular users can't create teams
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_team_creation_flow_nonexistent_owner() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());
    let app = test::init_service(client.create_app()).await;

    let (_admin_id, admin_token) = client.create_test_admin().await;

    // Use a non-existent user ID
    let fake_owner_id = uuid::Uuid::new_v4();
    let team_data = test_data::sample_team(fake_owner_id);

    let req = test::TestRequest::post()
        .uri("/team/create")
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(&team_data)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Should fail because owner doesn't exist
    assert!(resp.status().is_client_error() || resp.status().is_server_error());
}

#[tokio::test]
async fn test_team_invite_flow_success() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());
    let app = test::init_service(client.create_app()).await;

    // Create admin and set up team with owner
    let (_admin_id, admin_token) = client.create_test_admin().await;
    let (owner_id, owner_token) = client.create_test_user().await;
    let team_id = client.create_team_with_owner(owner_id).await;

    // Create target user to invite
    let target_user_email = "target@test.com";
    let target_user_data = test_data::sample_user_with_email(target_user_email);

    // Create the target user first
    let req_create_target = test::TestRequest::post()
        .uri("/user/create")
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(&target_user_data)
        .to_request();

    let resp_create = test::call_service(&app, req_create_target).await;
    assert_eq!(resp_create.status(), StatusCode::CREATED);

    // Now send invite
    let invite_data = RTeamInviteUser {
        user_email: target_user_email.to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/team/admin/invite")
        .insert_header(("Authorization", format!("Bearer {}", owner_token)))
        .set_json(&invite_data)
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["message"].as_str().unwrap().contains("sent an invite"));
}

#[tokio::test]
async fn test_team_invite_flow_not_owner() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());
    let app = test::init_service(client.create_app()).await;

    // Create admin and set up team with owner
    let (_admin_id, admin_token) = client.create_test_admin().await;
    let (owner_id, _owner_token) = client.create_test_user().await;
    let _team_id = client.create_team_with_owner(owner_id).await;

    // Create a different user (not team owner)
    let (non_owner_id, non_owner_token) = client.create_test_user().await;

    // Create target user to invite
    let target_user_email = "target2@test.com";
    let target_user_data = test_data::sample_user_with_email(target_user_email);

    let req_create_target = test::TestRequest::post()
        .uri("/user/create")
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(&target_user_data)
        .to_request();

    let resp_create = test::call_service(&app, req_create_target).await;
    assert_eq!(resp_create.status(), StatusCode::CREATED);

    // Try to send invite as non-owner
    let invite_data = RTeamInviteUser {
        user_email: target_user_email.to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/team/admin/invite")
        .insert_header(("Authorization", format!("Bearer {}", non_owner_token)))
        .set_json(&invite_data)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Should be forbidden since user is not team owner
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_team_invite_flow_target_user_not_found() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());
    let app = test::init_service(client.create_app()).await;

    // Create admin and set up team with owner
    let (_admin_id, _admin_token) = client.create_test_admin().await;
    let (owner_id, owner_token) = client.create_test_user().await;
    let _team_id = client.create_team_with_owner(owner_id).await;

    // Try to invite non-existent user
    let invite_data = RTeamInviteUser {
        user_email: "nonexistent@test.com".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/team/admin/invite")
        .insert_header(("Authorization", format!("Bearer {}", owner_token)))
        .set_json(&invite_data)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Should fail because target user doesn't exist
    assert!(resp.status().is_client_error() || resp.status().is_server_error());
}

#[tokio::test]
async fn test_team_invite_flow_invite_team_owner() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());
    let app = test::init_service(client.create_app()).await;

    // Create admin and set up team with owner
    let (_admin_id, admin_token) = client.create_test_admin().await;
    let (owner1_id, owner1_token) = client.create_test_user().await;
    let _team1_id = client.create_team_with_owner(owner1_id).await;

    // Create another team owner
    let (owner2_id, _owner2_token) = client.create_test_user().await;
    let _team2_id = client.create_team_with_owner(owner2_id).await;

    let owner2_email = ctx.db.get_user_by_id(&owner2_id).await.unwrap().email;

    // Try to invite another team owner
    let invite_data = RTeamInviteUser {
        user_email: owner2_email,
    };

    let req = test::TestRequest::post()
        .uri("/team/admin/invite")
        .insert_header(("Authorization", format!("Bearer {}", owner1_token)))
        .set_json(&invite_data)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Should be forbidden to invite existing team owners
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}
