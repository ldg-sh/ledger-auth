use actix_web::{post, web, HttpResponse, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use crate::{
    db::postgres_service::PostgresService,
    types::team::{RTeamAddUser, TeamAddUserRes},
    utils::token,
};

#[post("/add")]
pub async fn adduser(
    _req: actix_web::HttpRequest,
    db: web::Data<PostgresService>,
    data: web::Json<RTeamAddUser>,
    tok: BearerAuth,
) -> impl Responder {
    // 0) auth
    let issuer_id = match token::extract_token_parts(tok.token()) {
        Some(id) => id.0,
        None => return HttpResponse::Unauthorized().finish(),
    };

    // 1) issuer exists
    let issuer = match db.get_user_by_id(&issuer_id).await {
        Ok(i) => i,
        _ => return HttpResponse::Unauthorized().finish(),
    };

    // 2) payload sanity (dont be an idiot)
    if data.user == issuer.id {
        return HttpResponse::BadRequest().body("Cannot move yourself.");
    }

    // 3) target team exists & owned by issuer
    let target_team = match db.get_team(data.team).await {
        Ok(t) if t.owner == issuer.id => t,
        Ok(_) => return HttpResponse::Forbidden().body("Not owner of target team."),
        Err(_) => return HttpResponse::NotFound().body("Target team not found."),
    };

    // 4) target user exists
    let target_user = match db.get_user_by_id(&data.user).await {
        Ok(u) => u,
        Err(_) => return HttpResponse::NotFound().body("Target user not found."),
    };

    // 5) no-op if already member
    if target_user.team_id == data.team {
        return HttpResponse::Ok().json(TeamAddUserRes {
            message: "User already in team.".into(),
        });
    }

    // 6) issuer must own BOTH source and destination teams (prevents user stealing)
    let source_team = match db.get_team_for_user(target_user.id).await {
        Ok(t) => t,
        Err(_) => return HttpResponse::Conflict().body("Target user has no source team."),
    };
    if source_team.owner != issuer.id || target_team.owner != issuer.id {
        return HttpResponse::Forbidden().body("Must own both source and target teams.");
    }

    // 7) prevent moving a team owner unless they own the same source (already true) AND you allow ownership transfer
    if let Ok(true) = db.user_is_team_owner(target_user.id).await {
        // If they own *any* team other than source, block.
        if let Ok(false) = db.user_owns_team(target_user.id, source_team.id).await {
            return HttpResponse::Conflict().body("User owns another team. Transfer ownership first.");
        }
    }

    // 8) execute move
    if let Err(_) = db.set_user_team(data.user, data.team).await {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().json(TeamAddUserRes {
        message: format!("User {} has been added to team.", target_user.name),
    })
}
