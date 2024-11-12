use futures_util::TryStreamExt;
use mongodb::{bson::doc, error::Error, results::InsertOneResult, Collection, Database};
use crate::models::User;

pub struct UserRepository {
    collection: Collection<User>,
}

impl UserRepository {
    pub fn init(db: &Database) -> Self {
        let reg_states: Collection<User> = db.collection("users");
        UserRepository {
            collection: reg_states,
        }
    }
    pub async fn create_new_user(&self,user : &User) -> mongodb::error::Result<InsertOneResult> {
        self.collection.insert_one(user).await
    }
    pub async fn search_by_username(&self, username: &str) -> mongodb::error::Result<Option<User>> {
        let filter = doc! { "username": username };
        let result = self.collection.find_one(filter).await?;
        Ok(result)
    }
    pub async fn fetch_keys_for_user(&self, username: &str) -> Result<Vec<User>,Error> {
        let filter = doc! { "username": username };
                let cursor = self.collection.find(filter).await?;
                let users: Vec<User> = cursor
            .try_collect()
            .await?;
        Ok(users)
    }
    // pub async fn delete_by_username(&self, username: &str) -> mongodb::error::Result<u64> {
    //     let filter = doc! { "username": username };
    //     let result = self.collection.delete_one(filter).await?;
    //     Ok(result.deleted_count)
    // }
}
