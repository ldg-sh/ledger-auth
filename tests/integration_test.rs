// Integration test runner that includes all flow tests
// This ensures they all run with the same test database setup

mod common;

// Import all test modules
mod user_flows_test;
mod team_flows_test;
mod invite_accept_flow_test;
mod validation_flows_test;
mod health_flow_test;

use common::{TestContext, client::TestClient};

// Add a simple integration test to verify the setup works
#[tokio::test]
async fn test_integration_setup() {
    let ctx = TestContext::new().await;
    let client = TestClient::new(ctx.db.clone());
    
    // Basic test to ensure setup works by creating a user
    let (user_id, _token) = client.create_test_user().await;
    
    // Verify user exists in database
    let user = ctx.db.get_user_by_id(&user_id).await;
    assert!(user.is_ok());
}