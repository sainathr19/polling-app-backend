use actix_web::{body::MessageBody, dev::{ServiceRequest, ServiceResponse}, middleware::Next, HttpMessage};

use crate::helpers::jwt::JWT;


pub async fn jwt_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    println!("JWT middleware called");
    let req = req;

    // Extract the authorization token from the cookies
    let auth_token = req.cookie("authToken").map(|cookie| cookie.value().to_string());
    println!("{:?}",&auth_token);
    let jwt = JWT::init();
    match auth_token{
        Some(token)=>{
            match jwt.verify(&token) {
                Ok(true) => {
                    let claims = jwt.decode(&token);
                    match claims {
                        Ok(claim)=>{
                            println!("{:?}",&claim);
                            req.extensions_mut().insert(claim);
                            return next.call(req).await
                        },
                        Err(err)=>{
                            println!("{:?}",err);
                            return Err(actix_web::error::ErrorUnauthorized("Invalid token"));
                        }
                    }
                }
                _ => {
                    return Err(actix_web::error::ErrorUnauthorized("Invalid token"));
                }
            }
        },
        None =>{
            return Err(actix_web::error::ErrorUnauthorized("No token provided"));
        }
    }
}

