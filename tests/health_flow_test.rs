mod common;

use actix_web::{test, http::StatusCode};
use common::{TestContext, client::TestClient};

#[tokio::test]
async fn test_health_check_flow_success() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());
    
    let (_user_id, user_token) = client.create_test_user().await;
    
    let req = test::TestRequest::get()
        .uri("/health")
        .insert_header(("Authorization", format!("Bearer {}", user_token)))
        .to_request();
    
    let resp = test::call_service(&client.app, req).await;
    
    assert_eq!(resp.status(), StatusCode::OK);
    
    // Health endpoint should return empty response on success
    let body: serde_json::Value = test::read_body_json(resp).await;
    // Should be empty object or similar success indicator
}

#[tokio::test]
async fn test_health_check_flow_admin_token() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());
    
    let (_admin_id, admin_token) = client.create_test_admin().await;
    
    let req = test::TestRequest::get()
        .uri("/health")
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .to_request();
    
    let resp = test::call_service(&client.app, req).await;
    
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_health_check_flow_invalid_token() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());
    
    let req = test::TestRequest::get()
        .uri("/health")
        .insert_header(("Authorization", "Bearer invalid_token"))
        .to_request();
    
    let resp = test::call_service(&client.app, req).await;
    
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_health_check_flow_missing_auth() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());
    
    let req = test::TestRequest::get()
        .uri("/health")
        .to_request();
    
    let resp = test::call_service(&client.app, req).await;
    
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_health_check_flow_malformed_auth() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());
    
    let req = test::TestRequest::get()
        .uri("/health")
        .insert_header(("Authorization", "NotBearer token"))
        .to_request();
    
    let resp = test::call_service(&client.app, req).await;
    
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test] 
async fn test_health_check_flow_wrong_http_method() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());
    
    let (_user_id, user_token) = client.create_test_user().await;
    
    // Health endpoint expects GET, try POST
    let req = test::TestRequest::post()
        .uri("/health")
        .insert_header(("Authorization", format!("Bearer {}", user_token)))
        .to_request();
    
    let resp = test::call_service(&client.app, req).await;
    
    // Should return method not allowed
    assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
}