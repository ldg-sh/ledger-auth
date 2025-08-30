use tonic::{Request, Response, Status};
use super::pb::{
    authentication_server::{Authentication, AuthenticationServer},
    ValidationRequest, ValidationResponse,
};

#[derive(Clone, Default)]
pub struct AuthenticationSvc;

#[tonic::async_trait]
impl Authentication for AuthenticationSvc {
    async fn validate_authentication(
        &self,
        req: Request<ValidationRequest>,
    ) -> Result<Response<ValidationResponse>, Status> {
        let r = req.into_inner();
        println!("Got a req!!");

        // Example logic
        if r.token == "secret123" {
            Ok(Response::new(ValidationResponse {
                is_valid: true,
                message: format!("User {} authenticated", r.user_id),
            }))
        } else {
            Ok(Response::new(ValidationResponse {
                is_valid: false,
                message: "Invalid token".into(),
            }))
        }
    }
}

pub fn server() -> AuthenticationServer<AuthenticationSvc> {
    AuthenticationServer::new(AuthenticationSvc)
}
