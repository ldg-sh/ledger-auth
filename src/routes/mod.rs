use crate::utils::webutils::{validate_admin_token, validate_token};
use actix_web::web;

pub mod health;
pub mod validate;
pub mod user;
pub mod team;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    let user_auth = actix_web_httpauth::middleware::HttpAuthentication::bearer(validate_token);
    let admin_auth = actix_web_httpauth::middleware::HttpAuthentication::bearer(validate_admin_token);

    cfg.service(
        web::scope("/health").service(health::health).wrap(user_auth.clone())
    );
    cfg.service(
        web::scope("/user")
            .service(
                web::scope("/create")
                    .service(user::create::create)
                    .wrap(admin_auth.clone())
            )
            .service(
                web::scope("/regenerate")
                    .service(user::regenerate::regenerate)
                    .wrap(user_auth.clone())
            )
    );
    cfg.service(
        web::scope("/validate")
            .service(validate::validate)
    );
    cfg.service(
        web::scope("/team")
            .service(
                web::scope("/create")
                    .service(team::create::create_team)
                    .wrap(admin_auth.clone())
            )
            .service(
                web::scope("/user")
                    .service(team::adduser::adduser)
                    .wrap(admin_auth)
            )
    );// TODO: Auth for team routes
}
