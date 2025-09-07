use crate::utils::webutils::{validate_admin_token, validate_token};
use actix_web::web;

pub mod health;
pub mod validate;
pub mod user;
pub mod team;
pub mod fail;

     // TODO:
     // Obviously, some logic needs cleaning. We also need to create a middleware wrapper that checks if you are a team owner in general.
     // We need to find out if we can wrap twice in two types of authentication, which could be useful.



pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    let user_auth = actix_web_httpauth::middleware::HttpAuthentication::bearer(validate_token);
    //let admin_auth = actix_web_httpauth::middleware::HttpAuthentication::bearer(validate_admin_token);

    // Anything on the /health endpoint
    cfg.service(
        web::scope("/health").service(health::health)
    );

    // Anything in this .service block
    // is on the /user endpoint
    cfg.service(

        web::scope("/user")
            // user/create
            .service(
                web::scope("/create")
                    .service(user::create::create)
            )

            // user/regenerate
            .service(
                web::scope("/regenerate")
                    .service(user::regenerate::regenerate)
                    .wrap(user_auth.clone())
            )
    );

    // Anything on the /validate endpoint
    cfg.service(
        web::scope("/validate")
            .service(validate::validate)
    );

    // Anything on the /team endpoint
    cfg.service(
        web::scope("/team")
            // team/create
            .service(
                web::scope("/create")
                    .service(team::create::create_team)
                    .wrap(user_auth.clone())
            )

            // team/invite/accept
            .service(
                web::scope("/invite/accept")
                    .service(team::accept_invite::accept_invite)
                    .wrap(user_auth)
            )

            // team/admin
            .service(
                web::scope("/admin")
                    // team/admin/invite
                    .service(
                        web::scope("/invite")
                            .service(team::admin::invite::admin_invite)
                    )
            )

    );// TODO: Auth for team routes

    cfg.service(
        web::scope("/fail")
            .service(fail::fail)
    );
}
