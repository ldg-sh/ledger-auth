// use actix_web::{patch, HttpResponse};
// use crate::response;

// #[derive(Deserialize)]
// struct UserUpdate {
//     username: Option<String>,
//     email: Option<String>,
// }

// #[patch("/{id}")]
// async fn update(
//     req: actix_web::HttpRequest,
//     user_id: web::Path,
//     payload: web::Json<UserUpdate>,
// ) -> HttpResponse {
//     // logic

//     HttpResponse::Ok().json(
//         response::make_query_response(
//             true,
//             Some(format!("User {} updated successfully.", user_id)),
//             Some(payload),
//             Some("Update successful.")
//         )
//     )
// }
