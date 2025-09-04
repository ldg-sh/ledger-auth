use actix_web::{post, web::Json, HttpResponse};
use sea_orm::DbErr;
use serde::{Deserialize, Serialize};
use crate::types::{error::AppError, response::{ApiResponse, ApiResult}};

pub fn test_thing() -> Result<String, AppError> {
    let err = DbErr::RecordNotFound("Failed to find thingy".to_string());
    Err(AppError::Db(err))
}

#[derive(Serialize, Deserialize)]
pub struct TestRes {
    pub name: String,
    pub age: i64
}


#[post("")]
async fn fail(
    _req: actix_web::HttpRequest
) -> ApiResult<TestRes> {


    //test_thing()?;

    Ok(ApiResponse::Ok(TestRes { name: "Noah".to_string(), age: 18 }))
}
