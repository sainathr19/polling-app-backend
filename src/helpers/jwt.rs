use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    env,
    time::{SystemTime, UNIX_EPOCH},
};
use dotenv::dotenv;

// Define the claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub username: String,
    pub iss: String,
    pub sub: String,
    pub exp: u64,
}

pub struct JWT {
    secret: String,
}

impl JWT {
    pub fn init() -> Self {
        dotenv().ok();
        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        JWT { secret: jwt_secret }
    }

    pub fn sign(&self, username: String) -> Result<String, jsonwebtoken::errors::Error> {
        let now_plus_3600 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600;        
        let claims = Claims {
            username,
            iss: "http://www.example.com".to_owned(),
            sub: "name of claim".to_owned(),
            exp: now_plus_3600,
        };
        let header = Header::default();
        let secret = EncodingKey::from_secret(self.secret.as_bytes());
        let token = encode(&header, &claims, &secret)?;

        Ok(token)
    }

    pub fn decode(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let decoding_key = DecodingKey::from_secret(self.secret.as_bytes());
        let validation = Validation::default();
        let token_data = decode::<Claims>(&token, &decoding_key, &validation)?;

        Ok(token_data.claims)
    }

    pub fn verify(&self, token: &str) -> Result<bool, jsonwebtoken::errors::Error> {
        let decoding_key = DecodingKey::from_secret(self.secret.as_bytes());

        let validation = Validation::default();
        
        match decode::<Claims>(token, &decoding_key, &validation) {
            Ok(_) => Ok(true),
            Err(err) => match err.kind() {
                jsonwebtoken::errors::ErrorKind::InvalidAudience => Ok(false),
                _ => Err(err),
            },
        }
    }
}
