use actix_web::{get, HttpResponse};
use crate::response;

#[get("")]
async fn health(
    req: actix_web::HttpRequest
) -> HttpResponse {

    HttpResponse::Ok().json(
        response::make_query_response(
            true,
            Some(&"Endpoints are healthy!"),
            None,
            Some("Server is healthy!")
        )
    )
}
