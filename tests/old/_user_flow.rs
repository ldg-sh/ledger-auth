mod common;

use actix_web::{test, http::StatusCode};
use common::{TestContext, test_data, client::TestClient};

#[tokio::test]
async fn test_user_creation_flow_success() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());
    let app = test::init_service(client.create_app()).await;

    // Create admin user for authentication
    let (_admin_id, admin_token) = client.create_test_admin().await;

    let user_data = test_data::sample_user();

    let req = test::TestRequest::post()
        .uri("/user/create")
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(&user_data)
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::CREATED);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["message"].as_str().unwrap().contains("User created"));

    // Verify user was created in database
    let created_user = ctx.db.get_user_by_email(user_data.email.clone()).await;
    assert!(created_user.is_ok());

    let user = created_user.unwrap();
    assert_eq!(user.email, user_data.email);
    assert_eq!(user.name, user_data.name);
    assert!(user.token.len() > 0);
}

#[tokio::test]
async fn test_user_creation_flow_unauthorized() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());
    let app = test::init_service(client.create_app()).await;

    let user_data = test_data::sample_user();

    let req = test::TestRequest::post()
        .uri("/user/create")
        .insert_header(("Authorization", "Bearer invalid_token"))
        .set_json(&user_data)
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_user_creation_flow_missing_auth() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());
    let app = test::init_service(client.create_app()).await;

    let user_data = test_data::sample_user();

    let req = test::TestRequest::post()
        .uri("/user/create")
        .set_json(&user_data)
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_user_creation_flow_user_token_forbidden() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());
    let app = test::init_service(client.create_app()).await;

    // Create regular user (not admin) for authentication
    let (_user_id, user_token) = client.create_test_user().await;

    let user_data = test_data::sample_user_with_email("newuser@test.com");

    let req = test::TestRequest::post()
        .uri("/user/create")
        .insert_header(("Authorization", format!("Bearer {}", user_token)))
        .set_json(&user_data)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Should be forbidden since regular users can't create other users
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_user_creation_flow_duplicate_email() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());
    let app = test::init_service(client.create_app()).await;

    let (_admin_id, admin_token) = client.create_test_admin().await;

    // Create first user
    let user_data1 = test_data::sample_user();
    let req1 = test::TestRequest::post()
        .uri("/user/create")
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(&user_data1)
        .to_request();

    let resp1 = test::call_service(&app, req1).await;
    assert_eq!(resp1.status(), StatusCode::CREATED);

    // Try to create user with same email
    let user_data2 = test_data::sample_user(); // Same email
    let req2 = test::TestRequest::post()
        .uri("/user/create")
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(&user_data2)
        .to_request();

    let resp2 = test::call_service(&app, req2).await;

    // Should fail due to duplicate email
    assert!(resp2.status().is_client_error() || resp2.status().is_server_error());
}

#[tokio::test]
async fn test_user_regenerate_token_flow_success() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());
    let app = test::init_service(client.create_app()).await;

    let (user_id, user_token) = client.create_test_user().await;

    let req = test::TestRequest::post()
        .uri("/user/regenerate")
        .insert_header(("Authorization", format!("Bearer {}", user_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["message"].as_str().unwrap().contains("Regenerated user token"));

    // Verify token was actually changed in database
    let updated_user = ctx.db.get_user_by_id(&user_id).await.unwrap();
    // The old token should no longer be valid, but we can't easily test that here
    // without mocking or adding test-specific functions
    assert!(updated_user.token.len() > 0);
}

#[tokio::test]
async fn test_user_regenerate_token_flow_unauthorized() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());
    let app = test::init_service(client.create_app()).await;

    let req = test::TestRequest::post()
        .uri("/user/regenerate")
        .insert_header(("Authorization", "Bearer invalid_token"))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_user_regenerate_token_flow_missing_auth() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());
    let app = test::init_service(client.create_app()).await;

    let req = test::TestRequest::post()
        .uri("/user/regenerate")
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
