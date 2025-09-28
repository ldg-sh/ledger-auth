use crate::utils::token::{extract_token_parts, token_valid};
use actix_web::{dev::ServiceRequest, error::ErrorUnauthorized, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use std::sync::Arc;
use urlencoding;

use crate::{config::config, db::postgres_service::PostgresService};

pub fn decode_all(input: &str) -> Option<String> {
    urlencoding::decode(input).ok().map(|cow| cow.into_owned())
}

pub async fn validate_token(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    if credentials.token() == config().admin_key {
        Ok(req)
    } else {
        let db = match req.app_data::<web::Data<Arc<PostgresService>>>().cloned() {
            Some(db) => db,
            None => {
                return Err((
                    ErrorUnauthorized(
                        "DB unavailable. Please contact admin something bad happened.",
                    ),
                    req,
                ))
            }
        };

        if token_valid(&db, credentials.token()).await {
            return Ok(req);
        }

        Err((ErrorUnauthorized("Invalid token std validate"), req))
    }
}

/// For middleware to pass, you must own a team. Preliminary Auth.
pub async fn team_owner(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    if credentials.token() == config().admin_key {
        Ok(req)
    } else {
        let db = match req.app_data::<web::Data<Arc<PostgresService>>>().cloned() {
            Some(db) => db,
            None => {
                return Err((
                    ErrorUnauthorized(
                        "DB unavailable. Please contact admin something bad happened.",
                    ),
                    req,
                ))
            }
        };

        let user_id = match extract_token_parts(credentials.token()) {
            Some(info) => info.0,
            None => return Err((ErrorUnauthorized("Malformed auth token."), req)),
        };

        if db.user_is_team_owner(user_id).await.is_ok() {
            return Ok(req);
        }

        Err((
            ErrorUnauthorized("You must be a team owner to perform that action."),
            req,
        ))
    }
}

/// For middleware to pass, you must not own a team. Preliminary Auth.
pub async fn not_team_owner(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    if credentials.token() == config().admin_key {
        Ok(req)
    } else {
        let db = match req.app_data::<web::Data<Arc<PostgresService>>>().cloned() {
            Some(db) => db,
            None => {
                return Err((
                    ErrorUnauthorized(
                        "DB unavailable. Please contact admin something bad happened.",
                    ),
                    req,
                ))
            }
        };

        let user_id = match extract_token_parts(credentials.token()) {
            Some(info) => info.0,
            None => return Err((ErrorUnauthorized("Malformed auth token."), req)),
        };

        if db.user_is_team_owner(user_id).await.is_ok() {
            return Err((
                ErrorUnauthorized("You must be a team owner to perform that action."),
                req,
            ));
        }

        Ok(req)
    }
}

pub fn grpc_valid(tok: &str) -> bool {
    tok == config().grpc.auth_key
}

pub async fn validate_admin_token(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    // First check static admin key
    if credentials.token() == config().admin_key {
        return Ok(req);
    }

    // If not static admin key, check database admin tokens
    let db = match req.app_data::<web::Data<Arc<PostgresService>>>().cloned() {
        Some(db) => db,
        None => {
            return Err((
                ErrorUnauthorized("DB unavailable. Please contact admin something bad happened."),
                req,
            ))
        }
    };

    // Validate token exists in database
    if !token_valid(&db, credentials.token()).await {
        return Err((ErrorUnauthorized("Invalid token admin"), req));
    }

    // Check if token belongs to admin user by extracting user ID and checking token type
    let user_id = match extract_token_parts(credentials.token()) {
        Some(info) => info.0,
        None => return Err((ErrorUnauthorized("Malformed auth token."), req)),
    };

    // Get user from database and check if they have admin token
    match db.get_user_by_id(&user_id).await {
        Ok(_user) => {
            // Check if the stored token starts with admin_ prefix (this is the raw encrypted token)
            // We need to verify this is actually an admin token by checking if it was created with TokenType::Admin
            let (_, raw_token) = match extract_token_parts(credentials.token()) {
                Some(parts) => parts,
                None => return Err((ErrorUnauthorized("Malformed auth token."), req)),
            };

            if raw_token.starts_with("admin_") {
                Ok(req)
            } else {
                Err((ErrorUnauthorized("Admin privileges required"), req))
            }
        }
        Err(_) => Err((ErrorUnauthorized("User not found"), req)),
    }
}
