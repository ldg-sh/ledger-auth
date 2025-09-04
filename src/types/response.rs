use serde::Serialize;
use crate::types::error::AppError;
use actix_web::{HttpResponse, Responder};

pub enum ApiResponse<T> {
    Ok(T),
    EmptyOk,
    Created(T, String),
    NoContent,
}

impl<T: Serialize> Responder for ApiResponse<T> {
    type Body = actix_web::body::BoxBody;
    fn respond_to(self, _: &actix_web::HttpRequest) -> HttpResponse {
        match self {
            ApiResponse::Ok(v) => HttpResponse::Ok().json(v),
            ApiResponse::EmptyOk => HttpResponse::Ok().finish(),
            ApiResponse::Created(v, loc) => HttpResponse::Created()
                .append_header(("Location", loc))
                .json(v),
            ApiResponse::NoContent => HttpResponse::NoContent().finish(),
        }
    }
}

pub type ApiResult<T> = Result<ApiResponse<T>, AppError>;
