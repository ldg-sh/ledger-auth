use actix_web::{post, web, HttpResponse};
use std::sync::Arc;
use crate::{db::postgres_service::PostgresService, types::team::{RTeamCreate, TeamCreateRes}};


#[post("")]
async fn create_team(
    _req: actix_web::HttpRequest,
    db: web::Data<Arc<PostgresService>>,
    data: web::Json<RTeamCreate>
) -> HttpResponse {
    // TODO: Clean these values.
    let team = match db.create_team(data.owner, data.name.clone()).await {
        Ok(t) => t,
        Err(e) => {
            return HttpResponse::InternalServerError().body(e.to_string())
        },
    };

    match db.set_user_team(data.owner, team).await {
        Ok(_) => {},
        Err(_) => {
            return HttpResponse::InternalServerError().finish()
        },
    };

    HttpResponse::Ok().json(TeamCreateRes {
        id: team.to_string(),
        message: format!("Team {} has been successfully created.", data.name)
    })
}
