// use actix_web::{
//     dev::{Service, ServiceRequest, ServiceResponse, Transform},
//     error::ErrorUnauthorized,
//     Error, HttpMessage, HttpResponse, body::BoxBody,
// };
// use actix_web::web::Data;
// use futures_util::future::{ok, LocalBoxFuture, Ready};
// use std::task::{Context, Poll};

// use crate::helpers::jwt::JWT;

// // Custom middleware struct
// pub struct JwtMiddleware;

// impl<S, B> Transform<S, ServiceRequest> for JwtMiddleware
// where
//     S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
//     B: 'static,
// {
//     type Response = ServiceResponse<B>;
//     type Error = Error;
//     type Transform = JwtMiddlewareService<S>;
//     type InitError = ();
//     type Future = Ready<Result<Self::Transform, Self::InitError>>;

//     fn new_transform(&self, service: S) -> Self::Future {
//         ok(JwtMiddlewareService { service })
//     }
// }

// pub struct JwtMiddlewareService<S> {
//     service: S,
// }

// impl<S, B> Service<ServiceRequest> for JwtMiddlewareService<S>
// where
//     S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
//     B: 'static,
// {
//     type Response = ServiceResponse<B>;
//     type Error = Error;
//     type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

//     fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//         self.service.poll_ready(cx)
//     }

//     fn call(&self, req: ServiceRequest) -> Self::Future {
//         let protected_paths = vec!["/dashboard", "/profile", "/settings"];
//         let path = req.path().to_string();

//         let is_protected_route = protected_paths.iter().any(|p| path.starts_with(p));

//         if !is_protected_route {
//             return Box::pin(self.service.call(req));
//         }

//         let token = req.cookie("authToken").map(|cookie| cookie.value().to_string());

//         // Retrieve JWT service from request's app data
//         let jwt_service = req.app_data::<Data<JWT>>().cloned();

//         let fut = async move {
//             if let Some(token) = token {
//                 if let Some(jwt_service) = jwt_service {
//                     // Use the verify_and_decode method from MyJwtService
//                     match jwt_service.verify(&token) {
//                         Ok(decoded_data) => {
//                             req.extensions_mut().insert(decoded_data);
//                             let response = self.service.call(req).await?;

//                             // Ensure response is of type BoxBody
//                             let boxed_response = response.map_into_boxed_body();
//                             Ok(boxed_response)
//                         }
//                         Err(_) => {
//                             // Unauthorized response with BoxBody
//                             let unauthorized_response = HttpResponse::Unauthorized().finish();
//                             Ok(ServiceResponse::new(req, unauthorized_response.map_into_boxed_body()))
//                         }
//                     }
//                 } else {
//                     // Internal server error if JWT service is unavailable
//                     let error_response = HttpResponse::InternalServerError().finish();
//                     Ok(ServiceResponse::new(req, error_response.map_into_boxed_body()))
//                 }
//             } else {
//                 // Unauthorized response with BoxBody
//                 let unauthorized_response = HttpResponse::Unauthorized().finish();
//                 Ok(ServiceResponse::new(req, unauthorized_response.map_into_boxed_body()))
//             }
//         };

//         Box::pin(fut)
//     }
// }
