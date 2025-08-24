use actix_web::{dev::ServiceRequest, error::ErrorUnauthorized, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use urlencoding;
use std::sync::Arc;
use crate::utils::token::token_valid;

use crate::{config::config, db::postgres_service::PostgresService};

pub fn decode_all(input: &str) -> Option<String> {
    urlencoding::decode(input).ok().map(|cow| cow.into_owned())
}

pub async fn validate_token(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    if credentials.token() == config().admin_key {
        return Ok(req);
    } else {
        let db = match req.app_data::<web::Data<Arc<PostgresService>>>().cloned() {
            Some(db) => db,
            None => return Err((ErrorUnauthorized("DB unavailable. Please contact admin something bad happened."), req)),
        };

        if token_valid(&db, credentials.token()).await {
            return Ok(req)
        }

        return Err((ErrorUnauthorized("Invalid token").into(), req))
    }
}

pub async fn validate_admin_token(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    if credentials.token() == config().admin_key {
        return Ok(req);
    } else {
        return Err((ErrorUnauthorized("Invalid token").into(), req))
    }
}
