use crate::utils::token::{extract_token_parts, token_valid};
use actix_web::{dev::ServiceRequest, error::ErrorUnauthorized, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use std::sync::Arc;
use urlencoding;

use crate::{config::config, db::postgres_service::PostgresService};

pub fn decode_all(input: &str) -> Option<String> {
    urlencoding::decode(input).ok().map(|cow| cow.into_owned())
}

pub async fn validate_token(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    if credentials.token() == config().admin_key {
        Ok(req)
    } else {
        let db = match req.app_data::<web::Data<Arc<PostgresService>>>().cloned() {
            Some(db) => db,
            None => return Err((ErrorUnauthorized("DB unavailable. Please contact admin something bad happened."), req)),
        };

        if token_valid(&db, credentials.token()).await {
            return Ok(req)
        }

        Err((ErrorUnauthorized("Invalid token std validate"), req))
    }
}

/// For middleware to pass, you must own a team. Preliminary Auth.
pub async fn team_owner(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    if credentials.token() == config().admin_key {
        Ok(req)
    } else {
        let db = match req.app_data::<web::Data<Arc<PostgresService>>>().cloned() {
            Some(db) => db,
            None => return Err((ErrorUnauthorized("DB unavailable. Please contact admin something bad happened."), req)),
        };

        let user_id = match extract_token_parts(credentials.token()) {
            Some(info) => info.0,
            None => return Err((ErrorUnauthorized("Malformed auth token."), req)),
        };

        if db.user_is_team_owner(user_id).await.is_ok() {
            return Ok(req)
        }

        Err((ErrorUnauthorized("You must be a team owner to perform that action."), req))
    }
}

pub fn grpc_valid(tok: &str) -> bool {
    tok == config().grpc.auth_key
}

pub async fn validate_admin_token(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    if credentials.token() == config().admin_key {
        Ok(req)
    } else {
        Err((ErrorUnauthorized("Invalid token admin"), req))
    }
}
