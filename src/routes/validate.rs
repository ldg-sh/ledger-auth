use actix_web::post;
use serde::{Deserialize, Serialize};

use crate::types::response::{ApiResponse, ApiResult};

#[derive(Serialize, Deserialize)]
pub struct Response {}

#[post("")]
async fn validate(
    _req: actix_web::HttpRequest
) -> ApiResult<Response> {

    Ok(ApiResponse::EmptyOk)
}
