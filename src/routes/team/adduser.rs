use actix_web::{post, web, HttpResponse};
use std::sync::Arc;
use crate::{db::postgres_service::PostgresService, types::team::{RTeamAddUser, TeamAddUserRes}};


#[post("/add")]
async fn adduser(
    _req: actix_web::HttpRequest,
    db: web::Data<Arc<PostgresService>>,
    data: web::Json<RTeamAddUser>
) -> HttpResponse {
    // TODO: We still dont trust the user.
     match db.set_user_team(data.user, data.team).await {
        Ok(_) => {},
        Err(e) => {
            return HttpResponse::InternalServerError().body(e.to_string())
        },
    };

    let user = match db.get_user_by_id(&data.user).await {
        Ok(u) => u,
        Err(e) => {
            return HttpResponse::InternalServerError().body(e.to_string())
        },
    };


    HttpResponse::Ok().json(TeamAddUserRes {
        message: format!("User {} has been added to team.", user.name)
    })
}
