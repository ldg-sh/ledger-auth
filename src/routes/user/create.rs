use crate::config::config;
use crate::db::postgres_service::PostgresService;
use crate::types::token::TokenType;
use crate::types::user::UserCreateRes;
use crate::types::user::{DBUserCreate, RUserCreate};
use crate::utils::token::{construct_token, encrypt, encrypt_to_base64, new_token};
use actix_web::{post, web, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use std::sync::Arc;

#[post("/create")]
async fn create(
    _req: actix_web::HttpRequest,
    auth: BearerAuth,
    db: web::Data<Arc<PostgresService>>,
    body: web::Json<RUserCreate>,
) -> HttpResponse {
    if !(auth.token() == config().admin_key) {
        return HttpResponse::Unauthorized().finish()
    }
    // So the user passes uid.key. If they are an admin they just pass admin key
    // if they pass our user key this step will fail because the raw value is b64 not what we want.

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
