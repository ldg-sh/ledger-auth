use actix_web::post;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use sea_orm::DbErr;
use serde::{Deserialize, Serialize};
use crate::types::{error::AppError, response::{ApiResponse, ApiResult}};

pub fn test_thing() -> Result<String, AppError> {
    let err = DbErr::RecordNotFound("Failed to find thingy".to_string());
    Err(AppError::Db(err))
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub name: String,
    pub age: i64
}


#[post("")]
async fn fail(
    _req: actix_web::HttpRequest,
    b: BearerAuth
) -> ApiResult<Response> {


    //test_thing()?;

    Ok(ApiResponse::Ok(Response { name: "Noah".to_string(), age: 18 }))
}
