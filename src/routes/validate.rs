use actix_web::{post, Responder, HttpResponse};

use crate::types::{error::AppError, response::{ApiResponse, ApiResult}};

#[post("")]
async fn validate(
    _req: actix_web::HttpRequest
) -> ApiResult<String> {

    Ok(ApiResponse::EmptyOk)
}
