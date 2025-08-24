use std::sync::Arc;

use actix_web::{post, web, HttpResponse, HttpServer};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use uuid::Uuid;

use crate::{db::postgres_service::PostgresService, types::{mail::SendEmail, user::UserRegenerateTokenRes}, utils::{mail::send_email, token::{construct_token, decrypt_from_base64, extract_token_parts, token_valid}}};

#[post("")]
async fn regenerate(
    _req: actix_web::HttpRequest,
    db: web::Data<Arc<PostgresService>>,
    auth: BearerAuth
) -> HttpResponse {
    let user_id = match extract_token_parts(auth.token()) {
        Some(user_id) => {
            user_id.0
        },
        None => {
            return HttpResponse::Unauthorized().body("Invalid authorization token");
        },
    };

    let new_token = match db.regenerate_user_token(&user_id).await {
        Ok(new_token) => new_token,
        Err(e) => {
            return HttpResponse::InternalServerError().body(e.to_string())
        },
    };

    let user_email = match db.get_user_by_id(&user_id).await {
        Ok(user) => {
            user.email
        },
        Err(e) => {
            return HttpResponse::InternalServerError().body(e.to_string())
        },
    };


    let key = construct_token(&user_id, &new_token);

    let _ = send_email(SendEmail {
        from: "me@mail.noahdunnagan.com".to_string(),
        to: vec![user_email],
        subject: "Ledger access token reset.".to_string(),
        text: Some(format!("Your ledger access token has been reset. If this wasn't you, please contact support. \n \nYour new access key is: {}", key)),
        ..Default::default()
    }).await;

    HttpResponse::Ok().json(UserRegenerateTokenRes {
        message: "Regenerated user token, email has been sent with updated token.".to_string()
    })
}
