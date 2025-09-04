use actix_web::{post, web, HttpResponse, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use sea_orm::DbErr;
use crate::{db::postgres_service::PostgresService, utils::{mail::mail_token_reset, token::{construct_token, extract_token_parts}}};
use std::sync::Arc;

#[post("/{invite}")]
pub async fn accept_invite(
    _req: actix_web::HttpRequest,
    db: web::Data<Arc<PostgresService>>,
    path: web::Path<String>,
    auth: BearerAuth
) -> impl Responder {
    let inv_code = path.into_inner();

    let invite = match db.get_invite(&inv_code).await {
        Ok(i) => i,
        Err(DbErr::RecordNotFound(_)) => {
            return HttpResponse::NotFound().finish()
        },
        Err(_) => {
            return HttpResponse::InternalServerError().finish()
        },
    };

    let token_uid = match extract_token_parts(auth.token()) {
        Some(uid) => uid.0,
        None => {
            return HttpResponse::InternalServerError().body("Failed to extract token parts.")
        },
    };
    if token_uid != invite.user_id {
        return HttpResponse::Unauthorized().body("You arent allowed to accept an invite that isn't yours.")
    }

    match db.accept_invite(&invite.id).await {
        Ok(_) => {},
        Err(_) => {
            return HttpResponse::InternalServerError().finish()
        },
    };

    match db.set_user_team(invite.user_id, invite.team_id).await {
        Ok(_) => {},
        Err(_) => {
            return HttpResponse::InternalServerError().finish()
        },
    }

    let user_mail = match db.get_user_by_id(&invite.user_id).await {
        Ok(m) => m,
        Err(DbErr::RecordNotFound(_)) => {
            return HttpResponse::NotFound().body("Target user not found.")
        },
        Err(_) => {
            return HttpResponse::InternalServerError().finish()
        },
    };

    // Regenerate key because they are in a new team...
    let raw_token = match db.regenerate_user_token(&invite.user_id).await {
        Ok(t) => t,
        Err(_) => {
            return HttpResponse::InternalServerError().finish()
        },
    };

    // Construct and encode.
    let full_token = construct_token(&invite.user_id, &raw_token);

    mail_token_reset(&user_mail.email, &full_token).await.ok();

    HttpResponse::Ok().body("Successfully accepted invite and joined team!")
}
