 use actix_web::{post, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use chrono::{Duration, Utc};
 use std::sync::Arc;
 use crate::{db::postgres_service::PostgresService, types::{mail::SendEmail, team::RTeamInviteUser}, utils::{mail, token}};
 use crate::types::response::{ApiResponse, ApiResult};
 use crate::types::error::AppError;
 use serde::{Deserialize, Serialize};

/*
New system. Team owners create an "invite" for a user.
Users get emailed about this invite and are given a magic link and code.
To get moved, they have to accept and know that they will no longer have access to the old team.
team/join will have lots of notif checks too.
 */

 #[derive(Serialize, Deserialize)]
 pub struct Response {
     pub message: String,
 }

 #[post("")]
 async fn admin_invite(
     _req: actix_web::HttpRequest,
     db: web::Data<Arc<PostgresService>>,
     data: web::Json<RTeamInviteUser>,
     tok: BearerAuth
 ) -> ApiResult<Response> {
     let target_mail = data.0.user_email;

     let issuer_uid = match token::extract_token_parts(tok.token()) {
         Some(id) => id.0,
         None => return Err(AppError::Unauthorized),
     };

     // This is to check if the issuer exists
     let issuer = db.get_user_by_id(&issuer_uid).await?;

     let issuer_team_id = match issuer.team_id {
         Some(issuer_team_id) => issuer_team_id,
         None => return Err(AppError::Forbidden),
     };

     let team = db.get_team(issuer_team_id).await?;

     // Check if issuer is a team owner.
     let is_owner = db.user_is_team_owner(issuer_uid).await?;

     if !is_owner { return Err(AppError::Forbidden); }

     // Get the target user.
     let target = db.get_user_by_email(target_mail.clone()).await?;

     let target_is_owner = db.user_is_team_owner(target.id).await?;
     if target_is_owner { return Err(AppError::Forbidden); }

     let invite = db.create_invite(issuer_team_id, target.id, issuer_uid, Utc::now() + Duration::minutes(30)).await?;

     let _ = mail::send_email(SendEmail {
         from: "me@mail.noahdunnagan.com".to_string(),
         to: vec![target.email],
         subject: format!("{} team invite.", team.name),
         text: Some(format!("You have been invited to join {}. \n \nYour invite code is: {}", team.name, invite)),
         ..Default::default()
     }).await;

     Ok(ApiResponse::Ok(Response { message: "User has been sent an invite.".to_string() }))
 }
