use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::{env, time::{SystemTime, UNIX_EPOCH}};
use dotenv::dotenv;
// Define the claims structure
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    username: String,
    iss: String,
    sub: String,
    exp: u64,
    aud: String,
}

pub struct JWT {
    secret: String,
}

impl JWT {
    pub fn init() -> Self {
        dotenv().ok();
        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        JWT { secret : jwt_secret }
    }

    pub fn sign(&self, username: String) -> Result<String, jsonwebtoken::errors::Error> {
        let now_plus_3600 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600;
        let claims = Claims {
            username,
            iss: "http://www.example.com".to_owned(),
            sub: "name of claim".to_owned(),
            exp: now_plus_3600,
            aud: "John Smith".to_owned(),
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

        let mut validation = Validation::default();
        validation.set_audience(&vec!["John Smith"]);
        let token_data = decode::<Claims>(&token, &decoding_key, &validation);

        match token_data {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}
