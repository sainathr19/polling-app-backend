use std::collections::HashMap;

use actix_web::web::Data;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use webauthn_rs::prelude::*;

#[derive(Serialize,Deserialize,Debug)]
pub struct UserData {
    pub(crate) name_to_id: HashMap<String, Uuid>,
    pub(crate) keys: HashMap<Uuid, Vec<Passkey>>,
}

pub fn startup() -> (Data<Webauthn>, Data<Mutex<UserData>>) {
    let rp_id = "localhost";
    let rp_origin = Url::parse("http://localhost:3000").expect("Invalid URL");
    let builder = WebauthnBuilder::new(rp_id, &rp_origin).expect("Invalid configuration");
    let builder = builder.rp_name("Survey Sphere");
    let webauthn = Data::new(builder.build().expect("Invalid configuration"));
    let webauthn_users = Data::new(Mutex::new(UserData {
        name_to_id: HashMap::new(),
        keys: HashMap::new(),
    }));

    (webauthn, webauthn_users)
}