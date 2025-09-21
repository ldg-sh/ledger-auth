use super::pb::{authentication_server::{Authentication, AuthenticationServer}, FileStatusRequest, FileStatusResponse, GetUserTeamRequest, GetUserTeamResponse, ValidationRequest, ValidationResponse};
use crate::utils::token::{extract_token_parts, token_valid};
use crate::{config::config, db::postgres_service::PostgresService};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

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
        let header_token = req
            .metadata()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "None".to_string());

        let r = req.into_inner();

        let body_token_valid = token_valid(&self.pg, &r.token).await;

        let header_token_valid = header_token == config().grpc.auth_key;

        let ok = body_token_valid && header_token_valid;

        let user_id = extract_token_parts(&r.token).unwrap().0;

        Ok(Response::new(ValidationResponse {
            is_valid: ok,
            user_id: if ok { user_id.to_string() } else { "".into() },
            message: if ok { "ok".into() } else { "invalid".into() },
        }))
    }

    async fn get_file_status( // TODO: implement properly
        &self,
        req: Request<FileStatusRequest>,
    ) -> Result<Response<FileStatusResponse>, Status> {
        let _r = req.into_inner();

        Ok(Response::new(FileStatusResponse {
            status: 0,
            message: "".to_string(),
        }))
    }

    async fn get_user_team(
        &self,
        req: Request<GetUserTeamRequest>,
    ) -> Result<Response<GetUserTeamResponse>, Status> {
        let r = req.into_inner();
        let extracted_uuid = match Uuid::parse_str(&r.user_id) {
            Ok(uuid) => uuid,
            Err(_) => {
                return Err(Status::invalid_argument("Invalid UUID format for user_token"));
            }
        };

        let team = match self.pg.get_team_for_user(extracted_uuid).await {
            Ok(team) => team,
            Err(_) => {
                return Err(Status::not_found("User not found or no team associated"));
            }
        };

        Ok(Response::new(GetUserTeamResponse {
            user_team_id: team.id.to_string(),
            success: true
        }))
    }
}


pub fn server(pg: Arc<PostgresService>) -> AuthenticationServer<AuthenticationSvc> {
    AuthenticationServer::new(AuthenticationSvc::new(pg))
}
