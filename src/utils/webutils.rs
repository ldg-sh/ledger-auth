use actix_web::{dev::ServiceRequest, error::ErrorUnauthorized};
use actix_web_httpauth::extractors::{bearer::BearerAuth, AuthenticationError};
use urlencoding;

use crate::config::config;

pub fn decode_all(input: &str) -> Option<String> {
    urlencoding::decode(input).ok().map(|cow| cow.into_owned())
}

pub async fn validate_token(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    if credentials.token() == config().admin_key {
        return Ok(req);
    } else {
        return Err((ErrorUnauthorized("Invalid token").into(), req))
    }
}
