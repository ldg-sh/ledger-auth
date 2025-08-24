use crate::db::postgres_service::PostgresService;
use crate::types::token::TokenType;
use crate::types::user::UserCreateRes;
use crate::types::user::{DBUserCreate, RUserCreate};
use crate::utils::token::{construct_token, encrypt, new_token};
use actix_web::{post, web, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use std::sync::Arc;

#[post("")]
async fn create(
    _req: actix_web::HttpRequest,
    _auth: BearerAuth,
    db: web::Data<Arc<PostgresService>>,
    body: web::Json<RUserCreate>,
) -> HttpResponse {
    let token = new_token(TokenType::User);

    let encrypted_token = match encrypt(&token) {
        Ok(token) => token,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let user_id = match db
        .create_user(DBUserCreate {
            name: body.name.clone(),
            email: body.email.clone(),
            token: encrypted_token,
        })
        .await
    {
        Ok(user_id) => user_id,
        Err(e) => {
            return HttpResponse::InternalServerError().body(e.to_string())
        }
    };

    let access_token = construct_token(&user_id.to_string(), &token);

    HttpResponse::Ok().json(UserCreateRes {
        token: access_token
    })
}
