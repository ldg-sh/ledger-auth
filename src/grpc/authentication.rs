use std::sync::Arc;
use tonic::{Request, Response, Status};
use crate::db::postgres_service::PostgresService;
use super::pb::{
    authentication_server::{Authentication, AuthenticationServer},
    ValidationRequest, ValidationResponse,
};
use crate::utils::token::token_valid;

#[derive(Clone)]
pub struct AuthenticationSvc {
    pub pg: Arc<PostgresService>,
}

impl AuthenticationSvc {
    pub fn new(pg: Arc<PostgresService>) -> Self { Self { pg } }
}

#[tonic::async_trait]
impl Authentication for AuthenticationSvc {
    async fn validate_authentication(
        &self,
        req: Request<ValidationRequest>,
    ) -> Result<Response<ValidationResponse>, Status> {

        // Extract header token before consuming req
        let header_token = req.metadata().get("authorization")
            .and_then(|v| v.to_str().ok())
            .filter(|s| s.starts_with("Bearer "))
            .map(|s| s[7..].to_string()); // Remove "Bearer " prefix
            
        let r = req.into_inner();
        
        // Try to validate the token from the request body
        let body_token_valid = token_valid(&self.pg, &r.token).await;
        
        // Also check if authorization header has a valid token
        let header_token_valid = if let Some(token) = header_token {
            token_valid(&self.pg, &token).await
        } else {
            false
        };
        
        // Token is valid if either the body token or header token is valid
        let ok = body_token_valid || header_token_valid;

        Ok(Response::new(ValidationResponse {
            is_valid: ok,
            message: if ok { "ok".into() } else { "invalid".into() },
        }))
    }
}

pub fn server(pg: Arc<PostgresService>) -> AuthenticationServer<AuthenticationSvc> {
    AuthenticationServer::new(AuthenticationSvc::new(pg))
}
