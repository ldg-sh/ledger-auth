use crate::types::error::AppError;
use crate::types::response::{ApiResponse, ApiResult};
use crate::{
    db::postgres_service::PostgresService,
    utils::{
        mail::mail_token_reset,
        token::{construct_token, extract_token_parts},
    },
};
use actix_web::{post, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::error;

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub message: String,
}

#[post("/{invite}")]
pub async fn accept_invite(
    _req: actix_web::HttpRequest,
    db: web::Data<Arc<PostgresService>>,
    path: web::Path<String>,
    auth: BearerAuth,
) -> ApiResult<Response> {
    let inv_code = path.into_inner();

    let invite = db.get_invite(&inv_code).await?;

    let token_uid = match extract_token_parts(auth.token()) {
        Some(uid) => uid.0,
        None => {
            return Err(AppError::BadRequest(
                "Failed to extract token parts.".into(),
            ))
        }
    };
    if token_uid != invite.user_id {
        error!("Token UUID is NOT equal to the invitee user ID.");
        return Err(AppError::Unauthorized);
    }

    db.accept_invite(&invite.id).await?;

    db.set_user_team(invite.user_id, invite.team_id).await?;

    // TODO: Mail user about the new team join.
    let user_mail = db.get_user_by_id(&invite.user_id).await?;

    Ok(ApiResponse::Ok(Response {
        message: "Successfully accepted invite and joined team!".to_string(),
    }))
}
