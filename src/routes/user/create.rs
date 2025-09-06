use crate::db::postgres_service::PostgresService;
use crate::types::error::AppError;
use crate::types::response::{ApiResponse, ApiResult};
use crate::types::token::TokenType;
use crate::types::user::{DBUserCreate, RUserCreate};
use crate::utils::mail::mail_welcome;
use crate::utils::token::{construct_token, encrypt, new_token};
use actix_web::{post, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub message: String
}

#[post("")]
async fn create(
    _req: actix_web::HttpRequest,
    _auth: BearerAuth,
    db: web::Data<Arc<PostgresService>>,
    body: web::Json<RUserCreate>,
) -> ApiResult<Response> {
    // Authentication is handled by middleware
    let token = new_token(TokenType::User);

    let encrypted_token = match encrypt(&token) {
        Ok(token) => token,
        Err(_) => return Err(AppError::Internal("There was an issue while encrypting the user's token.".to_string())),
    };

    let user_id = db
        .create_user(DBUserCreate {
            name: body.name.clone(),
            email: body.email.clone(),
            token: encrypted_token,
        })
        .await?;

    let access_token = construct_token(&user_id, &token);

    mail_welcome(&body.email, &access_token).await.ok();

    let body = Response { message: "User created; token emailed.".to_string() };

    Ok(ApiResponse::Created(body))
}
