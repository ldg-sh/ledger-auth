use actix_web::{post, HttpResponse};

#[post("")]
async fn validate(
    _req: actix_web::HttpRequest
) -> HttpResponse {

    HttpResponse::Ok().body("Ok")
}
