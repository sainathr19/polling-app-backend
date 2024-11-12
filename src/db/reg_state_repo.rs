use mongodb::{bson::doc, results::InsertOneResult, Collection, Database};
use serde_json::Value;
use crate::models::UserRegistrationState;

pub struct RegStateRepository {
    collection: Collection<UserRegistrationState>,
}

impl RegStateRepository {
    pub fn init(db: &Database) -> Self {
        let reg_states: Collection<UserRegistrationState> = db.collection("reg_states");
        RegStateRepository {
            collection: reg_states,
        }
    }
    pub async fn insert_state(&self, username: &str, user_unique_id: &str, reg_state: Value) -> mongodb::error::Result<InsertOneResult> {
        let new_reg_state = UserRegistrationState {
            username: username.to_string(),
            user_unique_id: user_unique_id.to_string(),
            reg_state,
        };
        self.collection.insert_one(new_reg_state).await
    }
    pub async fn search_by_username(&self, username: &str) -> mongodb::error::Result<Option<UserRegistrationState>> {
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
