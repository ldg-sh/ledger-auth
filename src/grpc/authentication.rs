use std::sync::Arc;
use tonic::{Request, Response, Status};
use crate::{db::postgres_service::PostgresService, utils::webutils::grpc_valid};
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

        let auth = match req.metadata().get("authorization") {
            Some(v) => {
                v.to_str().unwrap_or("")
            },
            None => return Ok(Response::new(ValidationResponse { is_valid: false, message: "missing header".into()})),
        };

        if !grpc_valid(auth) {
            return Ok(Response::new(ValidationResponse { is_valid: false, message:"invalid token".into()}));
        }

        let r = req.into_inner();

        let ok = token_valid(&self.pg, &r.token).await;

        Ok(Response::new(ValidationResponse {
            is_valid: ok,
            message: if ok { "ok".into() } else { "invalid".into() },
        }))
    }
}

pub fn server(pg: Arc<PostgresService>) -> AuthenticationServer<AuthenticationSvc> {
    AuthenticationServer::new(AuthenticationSvc::new(pg))
}
