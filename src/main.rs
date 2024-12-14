mod db;
mod helpers;
mod middlewares;
mod models;
mod routes;

use actix_cors::Cors;
use actix_web::{
    get, http::header::CONTENT_TYPE, middleware::from_fn, web::{self, Data}, App, HttpResponse, HttpServer, Responder
};
use db::MongoDB;
use helpers::{jwt::JWT, poll_state::PollState, webauthn::startup};
use middlewares::auth_middleware::jwt_middleware;
use routes::poll_routes::{fetch_polls, live_poll_updates};

#[get("/")]
async fn greet() -> impl Responder {
    HttpResponse::Ok().body("Welcome to Backend")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize MongoDB connection
    let mongo_db = MongoDB::init().await;
    let mongo_db = Data::new(mongo_db);

    // Initialize WebAuthn
    let (webauthn, _) = startup();

    // Initialize JWT handler
    let jwt = Data::new(JWT::init());
    let poll_state = Data::new(PollState::new());

    // Start the HTTP server
    HttpServer::new(move || {
        App::new().wrap(
            Cors::default()
                .allowed_origin("http://54.234.229.76:3000")
                .allowed_methods(vec!["GET", "POST"])
                .allowed_headers(vec![CONTENT_TYPE])
                .expose_headers(["Set-Cookie"])
                .supports_credentials()
                .max_age(3600),
        )
            .app_data(webauthn.clone())
            .app_data(mongo_db.clone())
            .app_data(poll_state.clone())
            .app_data(jwt.clone()) 
            .service(greet)
            .service(web::scope("/auth").configure(routes::auth_routes::init))
            .service(live_poll_updates)
            .service(fetch_polls)
            .service(
                web::scope("")
                    .wrap(from_fn(jwt_middleware))
                    .service(web::scope("/polls").configure(routes::poll_routes::init))
            )
    })
    .bind(("0.0.0.0", 5000))?
    .run()
    .await
}
