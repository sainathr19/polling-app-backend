use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};
use dotenv::dotenv;

// Define the claims structure
#[derive(Debug, Serialize, Deserialize,Clone)]
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




#[cfg(test)]
mod tests {
    use super::*;

    fn setup_jwt() -> JWT {
        env::set_var("JWT_SECRET", "test_secret_key");
        JWT::init()
    }

    #[test]
    fn test_jwt_sign() {
        let jwt = setup_jwt();
        let result = jwt.sign("testuser".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_jwt_decode_valid_token() {
        let jwt = setup_jwt();
        let token = jwt.sign("testuser".to_string()).unwrap();
        let result = jwt.decode(&token);
        
        assert!(result.is_ok());
        let claims = result.unwrap();
        assert_eq!(claims.username, "testuser");
        assert_eq!(claims.iss, "http://www.example.com");
        assert_eq!(claims.sub, "name of claim");
    }

    #[test]
    fn test_jwt_verify_valid_token() {
        let jwt = setup_jwt();
        let token = jwt.sign("testuser".to_string()).unwrap();
        let result = jwt.verify(&token);
        
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_jwt_verify_invalid_token() {
        let jwt = setup_jwt();
        let result = jwt.verify("invalid.token.here");
        
        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_decode_invalid_token() {
        let jwt = setup_jwt();
        let result = jwt.decode("invalid.token.here");
        
        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_token_expiration() {
        let jwt = setup_jwt();
        let token = jwt.sign("testuser".to_string()).unwrap();
        assert!(jwt.verify(&token).unwrap());
    }

    #[test]
    fn test_different_secret_keys() {
        let jwt1 = setup_jwt();
        let token = jwt1.sign("testuser".to_string()).unwrap();        
        env::set_var("JWT_SECRET", "different_secret_key");
        let jwt2 = JWT::init();
        assert!(jwt2.verify(&token).is_err());
    }
}