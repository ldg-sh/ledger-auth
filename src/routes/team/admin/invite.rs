use crate::types::error::AppError;
use crate::types::response::{ApiResponse, ApiResult};
use crate::{
    db::postgres_service::PostgresService,
    types::team::RTeamInviteUser,
    utils::{mail::mail_team_invite, token},
};
use actix_web::{post, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use tracing::{error, info};

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
    tok: BearerAuth,
) -> ApiResult<Response> {
    let target_mail = data.clone().user_email;
    info!("got target mail");

    let issuer_uuid = match token::extract_token_parts(tok.token()) {
        Some(id) => id.0,
        None => {
            error!("Error while extracting token parts");
            return Err(AppError::Unauthorized);
        }
    };
    info!("Extracted token parts. {}", issuer_uuid);

    let _ = db.get_user_by_id(&issuer_uuid).await?; // ? returns if issuer doesnt exist.
    info!("Issuer existed, getting more info.");

    let target_team_uuid = match uuid::Uuid::from_str(&data.team_id) {
        Ok(t) => t,
        Err(_) => {
            return Err(AppError::BadRequest(
                "Invalid team ID. Failed UUID parse.".to_string(),
            ))
        }
    };
    info!("Got user by ID AND team UUID");

    let target_team = db.get_team(target_team_uuid).await?;

    info!("Got target team");
    // Is the admin the team owner.
    if issuer_uuid != target_team.owner {
        return Err(AppError::Forbidden);
    }

    // This is having an error and idk why.
    let target_team_userlist = match db.list_users_in_team(target_team.id).await {
        Ok(t) => t,
        Err(err) => {
            println!("{}", err);
            return Err(err)
        },
    };
    info!("Got team userlist");


    let target_user = db.get_user_by_email(&target_mail).await?;
    info!("Got user by email");

    // Is the user already in team?
    let already_in_team = target_team_userlist.iter().any(|u| u == &target_user.id);


    if already_in_team {
        return Err(AppError::AlreadyExists);
    }

    info!("Creating invite");
    let invite = db
        .create_invite(
            target_team_uuid,
            target_user.id,
            issuer_uuid,
            Utc::now() + Duration::minutes(30),
        )
        .await?;

    mail_team_invite(&target_mail, &target_team.name, &invite)
        .await
        .ok();

    Ok(ApiResponse::Ok(Response {
        message: "User has been sent an invite.".to_string(),
    }))
}
