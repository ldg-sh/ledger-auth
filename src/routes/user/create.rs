use actix_web::{post, HttpResponse};
use crate::response;
use crate::config::config;
use crate::utils::webutils;
use actix_web_httpauth::extractors::bearer::BearerAuth;

#[post("")]
async fn create(
    req: actix_web::HttpRequest,
    auth: BearerAuth
) -> HttpResponse {
    if key.is_none() {
        return HttpResponse::Unauthorized().finish()
    }
    if key.unwrap() != config().admin_key {
        return HttpResponse::Unauthorized().finish()
    }

    HttpResponse::Ok().json(
        response::make_query_response(
            true,
            Some(&"Endpoints are healthy!"),
            None,
            Some("Server is healthy!")
        )
    )
}
