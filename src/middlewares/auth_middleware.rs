use actix_web::{body::MessageBody, dev::{ServiceRequest, ServiceResponse}, middleware::Next, HttpMessage};

use crate::helpers::jwt::JWT;


pub async fn jwt_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    let req = req;

    let auth_token = req.cookie("authToken").map(|cookie| cookie.value().to_string());
    let jwt = JWT::init();
    if let Some(token) = auth_token {
        if jwt.verify(&token).map_err(|_| actix_web::error::ErrorUnauthorized("Invalid token")).is_ok() {
            if let Ok(claim) = jwt.decode(&token) {
                req.extensions_mut().insert(claim);
                return next.call(req).await;
            }
        }
    }
    Err(actix_web::error::ErrorUnauthorized("No token provided or Invalid token"))
}

