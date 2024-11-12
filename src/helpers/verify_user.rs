// use actix_web::{HttpRequest, HttpResponse};

// use super::jwt::JWT;

// pub async fn verify_user(req: HttpRequest) -> Result<HttpResponse> {
//     let jwt = JWT::new()
//         .map_err(|_| HttpResponse::InternalServerError().body("Failed to initialize JWT secret"))?;
//     let auth_token = req
//         .cookies()
//         .find(|c| c.name() == "authToken")
//         .map(|c| c.value().to_string());

//     if let Some(token) = auth_token {
//         match jwt.verify(&token) {
//             Ok(true) => {
//                 return Ok(HttpResponse::Ok().body("User verified"));
//             }
//             Ok(false) => {
//                 return Ok(HttpResponse::Unauthorized().body("Unauthorized"));
//             }
//             Err(_) => {
//                 return Ok(HttpResponse::Unauthorized().body("Unauthorized"));
//             }
//         }
//     }
//     Ok(HttpResponse::Unauthorized().body("Unauthorized"))
// }
