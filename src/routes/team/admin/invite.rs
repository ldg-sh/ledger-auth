 use actix_web::{post, web, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use chrono::{Duration, Utc};
use sea_orm::DbErr;
 use std::sync::Arc;
 use crate::{db::postgres_service::PostgresService, types::{mail::SendEmail, team::RTeamInviteUser}, utils::{mail, token}};

/*
New system. Team owners create an "invite" for a user.
Users get emailed about this invite and are given a magic link and code.
To get moved, they have to accept and know that they will no longer have access to the old team.
team/join will have lots of notif checks too.
 */

 #[post("")]
 async fn admin_invite(
     _req: actix_web::HttpRequest,
     db: web::Data<Arc<PostgresService>>,
     data: web::Json<RTeamInviteUser>,
     tok: BearerAuth
 ) -> HttpResponse {
     let target_mail = data.0.user_email;

     let issuer_uid = match token::extract_token_parts(tok.token()) {
         Some(id) => {
             id.0
         },
         None => {
             return HttpResponse::Unauthorized().body("Malformed token.")
         },
     };

     // This is to check if the issuer exists
     let issuer = match db.get_user_by_id(&issuer_uid).await {
         Ok(i) => i,
         Err(DbErr::RecordNotFound(_)) => {
             return HttpResponse::NotFound().body("User not found.")
         },
         Err(e) => {
             eprintln!("admin_invite: get_user_by_id error: {}", e);
             return HttpResponse::InternalServerError().body("Error while getting user.")
         },
     };

     let issuer_team_id = match issuer.team_id {
         Some(issuer_team_id) => issuer_team_id,
         None => {
             return HttpResponse::Unauthorized().body("You are not a part of a team")
         },
     };

     let team = match db.get_team(issuer_team_id).await {
         Ok(t) => t,
         Err(DbErr::RecordNotFound(_)) => {
             return HttpResponse::NotFound().body("This team does not exist.")
         },
         Err(e) => {
             eprintln!("admin_invite: get_team error: {}", e);
             return HttpResponse::InternalServerError().finish()
         },
     };

     // Check if issuer is a team owner.
     let is_owner = match db.user_is_team_owner(issuer_uid).await {
         Ok(b) => b,
         Err(e) => {
             eprintln!("admin_invite: user_is_team_owner (issuer) error: {}", e);
             return HttpResponse::InternalServerError().finish()
         },
     };

     if !is_owner {
         return HttpResponse::Unauthorized().body("You are not allowed to perform that action.")
     }

     // Get the target user.
     let target = match db.get_user_by_email(target_mail.clone()).await {
         Ok(t) => t,
         Err(DbErr::RecordNotFound(_)) => {
           return HttpResponse::NotFound().body("Target user not found.")
         },
         Err(e) => { // Temp!
             eprintln!("admin_invite: get_user_by_email error: {}", e);
             return HttpResponse::InternalServerError().finish()
         },
     };

     match db.user_is_team_owner(target.id).await {
         Ok(is_owner) => {
             if is_owner {
                return HttpResponse::Unauthorized().body("You are not allowed to perform that action. Target is a team owner.")
             }
         },
         Err(e) => {
             eprintln!("admin_invite: user_is_team_owner (target) error: {}", e);
             return HttpResponse::InternalServerError().finish()
         },
     }

     let invite = match db.create_invite(issuer_team_id, target.id, issuer_uid, Utc::now() + Duration::minutes(30)).await {
         Ok(i) => i,
         Err(e) => {
             eprintln!("admin_invite: create_invite error: {}", e);
             return HttpResponse::InternalServerError().finish()
         },
     };

     let _ = mail::send_email(SendEmail {
         from: "me@mail.noahdunnagan.com".to_string(),
         to: vec![target.email],
         subject: format!("{} team invite.", team.name),
         text: Some(format!("You have been invited to join {}. \n \nYour invite code is: {}", team.name, invite)),
         ..Default::default()
     }).await;

     HttpResponse::Ok().body("User has been sent an invite.")
 }
