use crate::utils::webutils::{validate_admin_token, validate_token};
use actix_web::web;

pub mod health;
pub mod user;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    let _user_auth = actix_web_httpauth::middleware::HttpAuthentication::bearer(validate_token);
    let admin_auth = actix_web_httpauth::middleware::HttpAuthentication::bearer(validate_admin_token);

    cfg.service(
        web::scope("/health").service(health::health)
    );
    cfg.service(
        web::scope("/user/")
            .service(
                web::scope("/create")
                    .service(user::create::create)
                    .wrap(admin_auth)
            )
    );
}
