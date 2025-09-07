mod common;

use actix_web::{test, http::StatusCode};
use common::{TestContext, client::TestClient};

#[tokio::test]
async fn test_health_check_flow_success() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());
    let app = test::init_service(client.create_app()).await;

    let req = test::TestRequest::get()
        .uri("/health")
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_health_check_flow_wrong_http_method() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());
    let app = test::init_service(client.create_app()).await;

    let (_user_id, user_token) = match client.create_test_user(None).await {
        Ok(i) => i,
        Err(e) => {
            panic!("Failed creating a test user. \n\n E: {}", e)
        },
    };

    // Health endpoint expects GET, try POST
    let req = test::TestRequest::post()
        .uri("/health")
        .insert_header(("Authorization", format!("Bearer {}", user_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Should return not found.
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
