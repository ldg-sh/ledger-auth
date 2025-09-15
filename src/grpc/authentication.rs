use std::{os::unix::raw, sync::Arc};
use tonic::{Request, Response, Status};
use crate::{config::config, db::postgres_service::PostgresService};
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
        println!("Hit gRPC");

        // Extract header token before consuming req
        let header_token = req
            .metadata()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "None".to_string());

        let r = req.into_inner();

        // Try to validate the token from the request body
        let body_token_valid = token_valid(&self.pg, &r.token).await;

        // Also check if authorization header has a valid token
        let header_token_valid = header_token == config().grpc.auth_key;

        // Token is valid if the body token and header token are valid
        let ok = body_token_valid && header_token_valid;

        Ok(Response::new(ValidationResponse {
            is_valid: ok,
            message: if ok { "ok".into() } else { "invalid".into() },
        }))
    }
}

pub fn server(pg: Arc<PostgresService>) -> AuthenticationServer<AuthenticationSvc> {
    AuthenticationServer::new(AuthenticationSvc::new(pg))
}
