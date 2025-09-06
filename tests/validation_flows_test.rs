#[path = "common/mod.rs"]
mod common;

use actix_web::{test, http::StatusCode};
use common::{TestContext, client::TestClient};
use tonic::Request;
use ledger_auth::grpc::pb::authentication_server::Authentication;

// HTTP validation tests
#[tokio::test]
async fn test_http_token_validation_flow_success() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());
    let app = test::init_service(client.create_app()).await;
    
    let (_user_id, user_token) = client.create_test_user().await;
    
    let req = test::TestRequest::post()
        .uri("/validate")
        .insert_header(("Authorization", format!("Bearer {}", user_token)))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    assert_eq!(resp.status(), StatusCode::OK);
    
    // The validate endpoint returns empty response on success
    let _body: serde_json::Value = test::read_body_json(resp).await;
    // Should be empty object or similar success indicator
}

#[tokio::test]
async fn test_http_token_validation_flow_invalid_token() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());
    let app = test::init_service(client.create_app()).await;
    
    let req = test::TestRequest::post()
        .uri("/validate")
        .insert_header(("Authorization", "Bearer invalid_token_here"))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_http_token_validation_flow_missing_auth() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());
    let app = test::init_service(client.create_app()).await;
    
    let req = test::TestRequest::post()
        .uri("/validate")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_http_token_validation_flow_malformed_auth() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());
    let app = test::init_service(client.create_app()).await;
    
    let req = test::TestRequest::post()
        .uri("/validate")
        .insert_header(("Authorization", "NotBearer invalid_token"))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// gRPC validation tests
#[tokio::test]
async fn test_grpc_token_validation_flow_success() {
    let ctx = TestContext::new().await;
    
    // Create test user and get token
    let (_user_id, user_token) = {
        let client = TestClient::new(ctx.db.clone());
        client.create_test_user().await
    };
    
    // Create gRPC service
    let auth_svc = ledger_auth::grpc::authentication::AuthenticationSvc::new(ctx.db.clone());
    
    // Create request with metadata
    let mut request = Request::new(ledger_auth::grpc::pb::ValidationRequest {
        token: user_token.clone(),
    });
    
    // Add authorization header (gRPC expects "Bearer token" format)
    request.metadata_mut().insert(
        "authorization",
        format!("Bearer {}", user_token).parse().unwrap(),
    );
    
    let response = auth_svc.validate_authentication(request).await;
    
    assert!(response.is_ok());
    let resp = response.unwrap();
    let validation_response = resp.into_inner();
    
    assert!(validation_response.is_valid);
    assert_eq!(validation_response.message, "ok");
}

#[tokio::test]
async fn test_grpc_token_validation_flow_invalid_token() {
    let ctx = TestContext::new().await;
    
    let auth_svc = ledger_auth::grpc::authentication::AuthenticationSvc::new(ctx.db.clone());
    
    let mut request = Request::new(ledger_auth::grpc::pb::ValidationRequest {
        token: "invalid_token".to_string(),
    });
    
    request.metadata_mut().insert(
        "authorization",
        "Bearer invalid_token".parse().unwrap(),
    );
    
    let response = auth_svc.validate_authentication(request).await;
    
    assert!(response.is_ok());
    let resp = response.unwrap();
    let validation_response = resp.into_inner();
    
    assert!(!validation_response.is_valid);
    assert_eq!(validation_response.message, "invalid token");
}

#[tokio::test]
async fn test_grpc_token_validation_flow_missing_auth_header() {
    let ctx = TestContext::new().await;
    
    let auth_svc = ledger_auth::grpc::authentication::AuthenticationSvc::new(ctx.db.clone());
    
    // Request without authorization header
    let request = Request::new(ledger_auth::grpc::pb::ValidationRequest {
        token: "some_token".to_string(),
    });
    
    let response = auth_svc.validate_authentication(request).await;
    
    assert!(response.is_ok());
    let resp = response.unwrap();
    let validation_response = resp.into_inner();
    
    assert!(!validation_response.is_valid);
    assert_eq!(validation_response.message, "missing header");
}

#[tokio::test]
async fn test_grpc_token_validation_flow_malformed_auth_header() {
    let ctx = TestContext::new().await;
    
    let auth_svc = ledger_auth::grpc::authentication::AuthenticationSvc::new(ctx.db.clone());
    
    let mut request = Request::new(ledger_auth::grpc::pb::ValidationRequest {
        token: "some_token".to_string(),
    });
    
    // Malformed auth header (not "Bearer token" format)
    request.metadata_mut().insert(
        "authorization",
        "NotBearer some_token".parse().unwrap(),
    );
    
    let response = auth_svc.validate_authentication(request).await;
    
    assert!(response.is_ok());
    let resp = response.unwrap();
    let validation_response = resp.into_inner();
    
    assert!(!validation_response.is_valid);
    assert_eq!(validation_response.message, "invalid token");
}

#[tokio::test]
async fn test_grpc_token_validation_flow_admin_token() {
    let ctx = TestContext::new().await;
    
    // Create test admin and get token
    let (_admin_id, admin_token) = {
        let client = TestClient::new(ctx.db.clone());
        client.create_test_admin().await
    };
    
    let auth_svc = ledger_auth::grpc::authentication::AuthenticationSvc::new(ctx.db.clone());
    
    let mut request = Request::new(ledger_auth::grpc::pb::ValidationRequest {
        token: admin_token.clone(),
    });
    
    request.metadata_mut().insert(
        "authorization",
        format!("Bearer {}", admin_token).parse().unwrap(),
    );
    
    let response = auth_svc.validate_authentication(request).await;
    
    assert!(response.is_ok());
    let resp = response.unwrap();
    let validation_response = resp.into_inner();
    
    assert!(validation_response.is_valid);
    assert_eq!(validation_response.message, "ok");
}

#[tokio::test] 
async fn test_grpc_token_validation_flow_token_mismatch() {
    let ctx = TestContext::new().await;
    
    // Create test user and get token
    let (_user_id, user_token) = {
        let client = TestClient::new(ctx.db.clone());
        client.create_test_user().await
    };
    
    let auth_svc = ledger_auth::grpc::authentication::AuthenticationSvc::new(ctx.db.clone());
    
    let mut request = Request::new(ledger_auth::grpc::pb::ValidationRequest {
        token: "different_token".to_string(), // Different from auth header
    });
    
    request.metadata_mut().insert(
        "authorization",
        format!("Bearer {}", user_token).parse().unwrap(),
    );
    
    let response = auth_svc.validate_authentication(request).await;
    
    assert!(response.is_ok());
    let resp = response.unwrap();
    let validation_response = resp.into_inner();
    
    // Should be invalid because the token in the request body doesn't match the auth header
    assert!(!validation_response.is_valid);
    assert_eq!(validation_response.message, "invalid");
}
