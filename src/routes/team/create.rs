use actix_web::{post, web};
use std::sync::Arc;
use crate::{db::postgres_service::PostgresService, types::team::RTeamCreate};
use crate::types::response::{ApiResponse, ApiResult};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub id: String,
    pub message: String,
}


#[post("")]
async fn create_team(
    _req: actix_web::HttpRequest,
    db: web::Data<Arc<PostgresService>>,
    data: web::Json<RTeamCreate>
) -> ApiResult<Response> {
    // TODO: Clean these values.
    let team = db.create_team(data.owner, data.name.clone()).await?;

    db.set_user_team(data.owner, team).await?;

    Ok(ApiResponse::Ok(Response {
        id: team.to_string(),
        message: format!("Team {} has been successfully created.", data.name),
    }))
}
