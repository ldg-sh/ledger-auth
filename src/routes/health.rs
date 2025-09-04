use actix_web::get;
use serde::{Deserialize, Serialize};

use crate::types::response::{ApiResponse, ApiResult};

#[derive(Serialize, Deserialize)]
pub struct Response {}

#[get("")]
async fn health(
    _req: actix_web::HttpRequest
) -> ApiResult<Response> {
    Ok(ApiResponse::EmptyOk)
}
