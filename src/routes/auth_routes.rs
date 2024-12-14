use actix_session::{SessionGetError, SessionInsertError};
use actix_web::cookie::time::Duration;
use actix_web::cookie::{Cookie, SameSite};
use actix_web::{get, http::StatusCode};
use actix_web::{web, HttpRequest};
use uuid::Uuid;
use actix_web::{
    post, web::{Data, Json, Path}, HttpResponse, Responder
};
use webauthn_rs::prelude::*;
use thiserror::Error;
use crate::db::MongoDB;
use crate::helpers::jwt::JWT;
use crate::models:: User;


#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("Corrupt get session error")]
    SessionGet(#[from] SessionGetError),
    #[error("Corrupt insert session error")]
    SessionInsert(#[from] SessionInsertError),
    #[error("Bad request")]
    BadRequest(#[from] WebauthnError),
}

impl actix_web::ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[get("/verify")]
async fn verify_auth_token(req: HttpRequest , jwt : Data<JWT>) -> impl Responder {
    match req.cookie("authToken") {
        Some(cookie) => {
            let token = cookie.value();
            match jwt.decode(token) {
                Ok(val) => {
                        return HttpResponse::Ok().body(val.username)
                },
                Err(_) => HttpResponse::Unauthorized().body("Invalid or expired token"),
            }
        }
        None => HttpResponse::BadRequest().body("authToken cookie not found"),
    }
}

#[get("/logout")]
async fn logout() -> impl Responder {

    let mut response = HttpResponse::Ok().body("Logged out successfully");

    let cookie = Cookie::build("authToken", "")
            .http_only(true)
            .secure(true)
            .path("/")  
            .max_age(Duration::seconds(0))
            .finish();

    response.add_cookie(&cookie).unwrap();

    response
}

#[post("/register/start/{username}")]
pub async fn start_registration(
    mongo_db: web::Data<MongoDB>, 
    username: Path<String>,
    webauthn: web::Data<Webauthn>,
) -> impl Responder {
    let username_stat = mongo_db.user_collection.search_by_username(&username).await;
    let user_unique_id = match username_stat{
        Ok(user)=>{
            match user{

                Some(_)=> return HttpResponse::BadGateway().json("Username Already Exists"),
                None=>Uuid::new_v4()
            }
        },
        Err(err)=>{
            println!("{:?}",err);
            return HttpResponse::InternalServerError().json("Error Validating Username");
        }
    };
    let (ccr, reg_state) = match webauthn.start_passkey_registration(
        Uuid::new_v4(),
        &username,
        &username,
        None,
    ) {
        Ok(result) => result,
        Err(e) => return HttpResponse::BadRequest().body(format!("Registration start error: {}", e)),
    };

    let reg_state_value = match serde_json::to_value(&reg_state) {
        Ok(value) => value,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to serialize reg_state: {}", e)),
    };
    let reg_state_repo = &mongo_db.reg_state_collection;
    match reg_state_repo.insert_state(&username, &user_unique_id.to_string(), reg_state_value).await {
        Ok(_) => HttpResponse::Ok().json(ccr),
        Err(e) => {
            println!("{:?}", e);
            HttpResponse::InternalServerError().json("Unable to insert registration state into MongoDB")
        }
    }
}

#[post("/register/finish/{username}")]
pub async fn finish_registration(
    mongo_db: web::Data<MongoDB>,
    req: Json<RegisterPublicKeyCredential>,
    username: Path<String>,
    webauthn: web::Data<Webauthn>,
) -> impl Responder {
    let reg_state_repo = &mongo_db.reg_state_collection;
    let reg_state = match reg_state_repo.search_by_username(&username).await {
        Ok(Some(state)) => state,
        Ok(None) => return HttpResponse::BadRequest().json("No registration state found for username"),
        Err(e) => return HttpResponse::InternalServerError().body(format!("Error fetching reg state: {}", e)),
    };
    let passkey_registration: PasskeyRegistration = match serde_json::from_value(reg_state.reg_state) {
        Ok(reg) => reg,
        Err(e) => return HttpResponse::BadRequest().body(format!("Failed to deserialize reg_state: {}", e)),
    };
    let sk: Passkey = match webauthn.finish_passkey_registration(&req, &passkey_registration) {
        Ok(result) => result,
        Err(e) => return HttpResponse::BadRequest().body(format!("Registration finish error: {}", e)),
    };

    let sk_value = match serde_json::to_value(&sk) {
        Ok(value) => value,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to serialize sk: {}", e)),
    };
    let user = User {
        username: username.clone(),
        sk : sk_value
    };
    
    let user_repo = &mongo_db.user_collection;
    match user_repo.create_new_user(&user).await {
        Ok(_) => println!("Successfully created new user: {}", username),
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Error creating new user: {}", e))
        }
    }

    match reg_state_repo.delete_by_username(&username).await {
        Ok(_) => println!("Successfully deleted reg_state for username: {}", username),
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Error deleting reg_state: {}", e))
        }
    }
    HttpResponse::Ok().finish()
}


#[post("/authenticate/start/{username}")]
pub async fn start_authentication(
    mongo_db: web::Data<MongoDB>, 
    username: Path<String>, 
    webauthn: web::Data<Webauthn>
) -> impl Responder {
    // Fetch user unique ID from MongoDB
    let _ = match mongo_db.user_collection.search_by_username(&username).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return HttpResponse::NotFound().body("User not found");
        },
        Err(_) => {
            return HttpResponse::InternalServerError().body("Failed to fetch user ID");
        }
    };
    
    // Fetching the passkeys of user from the database
    let user_keys = match mongo_db.user_collection.fetch_keys_for_user(&username).await{
        Ok(val)=>val,
        Err(err)=>{
            println!("{:?}",err);
            return HttpResponse::InternalServerError().body("Error fetching user credentials")
        }
    };
    let mut creds: Vec<Passkey> = Vec::new();
    for user in user_keys{
        let credential: Passkey = match serde_json::from_value(user.sk) {
            Ok(val)=>val,
            Err(err)=>{
                println!("{:?}",err);
                return HttpResponse::BadRequest().json("User Keys Corrupted,Unable to Deserialize")
            }  
        };
        creds.push(credential);
    }

    let (rcr, auth_state) = match webauthn.start_passkey_authentication(&creds) {
        Ok(result) => result,
        Err(e) => {
            return HttpResponse::BadRequest().body(format!("Authentication start error: {}", e));
        }
    };
    // Serialize the authentication state
    let auth_state_value = match serde_json::to_value(&auth_state) {
        Ok(value) => value,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Failed to serialize reg_state: {}", e));
        }
    };

    // Store the authentication state in MongoDB
    let auth_state_repo = &mongo_db.auth_state_collection;
    match auth_state_repo.insert_state(&username, auth_state_value).await {
        Ok(_) => HttpResponse::Ok().json(rcr),
        Err(e) => {
            println!("{:?}", e);
            HttpResponse::InternalServerError().json("Unable to insert registration state into MongoDB")
        }
    }
}


#[post("/authenticate/finish/{username}")]
pub async fn finish_authentication(
    mongo_db : Data<MongoDB>,
    jwt : Data<JWT>,
    auth: Json<PublicKeyCredential>, 
    username : Path<String>,
    webauthn: web::Data<Webauthn>
) -> impl Responder {
    let auth_state_repo = &mongo_db.auth_state_collection;
    let auth_state = match auth_state_repo.search_by_username(&username).await {
        Ok(Some(state)) => state,
        Ok(None) => return HttpResponse::BadRequest().json("No Authentication state found for username"),
        Err(e) => return HttpResponse::InternalServerError().body(format!("Error fetching reg state: {}", e)),
    };
    let passkey_authentication: PasskeyAuthentication = match serde_json::from_value(auth_state.auth_state) {
        Ok(reg) => reg,
        Err(e) => return HttpResponse::BadRequest().body(format!("Failed to deserialize auth_state: {}", e)),
    };
    let _ = match webauthn.finish_passkey_authentication(&auth, &passkey_authentication) {
        Ok(result) => result,
        Err(e) => return HttpResponse::BadRequest().body(format!("Authentication finish error: {}", e)),
    };
    match auth_state_repo.delete_by_username(&username).await {
        Ok(_) => println!("Successfully deleted auth_state for username: {}", username),
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Error deleting auth_state: {}", e))
        }
    }

    // Create JWT token
    let token = match jwt.sign(username.into_inner()) {
        Ok(t) => t,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to create JWT: {}", e)),
    };

    let auth_cookie = Cookie::build("authToken", token)
    .http_only(true).secure(false)
    .same_site(SameSite::None).secure(true)
    .path("/")
    .max_age(Duration::seconds(3600))
    .finish();


    //Include Jwt token in cookie
    HttpResponse::Ok()
        .cookie(auth_cookie)
        .body("Authentication successful")
}


pub fn init(config: &mut web::ServiceConfig) {
    config
        .service(start_registration)
        .service(finish_registration)
        .service(start_authentication)
        .service(finish_authentication)
        .service(verify_auth_token)
        .service(logout);
} 