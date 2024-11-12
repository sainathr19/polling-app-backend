use mongodb::{bson::doc, results::InsertOneResult, Collection, Database};
use serde_json::Value;
use crate::models::UserAuthenticationState;

pub struct AuthStateRepository {
    collection: Collection<UserAuthenticationState>,
}

impl AuthStateRepository {
    pub fn init(db: &Database) -> Self {
        let auth_states: Collection<UserAuthenticationState> = db.collection("auth_states");
        AuthStateRepository {
            collection: auth_states,
        }
    }
    pub async fn insert_state(&self, username: &str, auth_state: Value) -> mongodb::error::Result<InsertOneResult> {
        let new_reg_state = UserAuthenticationState {
            username: username.to_string(),
            auth_state,
        };
        self.collection.insert_one(new_reg_state).await
    }
    pub async fn search_by_username(&self, username: &str) -> mongodb::error::Result<Option<UserAuthenticationState>> {
        let filter = doc! { "username": username };
        let result = self.collection.find_one(filter).await?;
        Ok(result)
    }
    pub async fn delete_by_username(&self, username: &str) -> mongodb::error::Result<u64> {
        let filter = doc! { "username": username };
        let result = self.collection.delete_one(filter).await?;
        Ok(result.deleted_count)
    }
}
