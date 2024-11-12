use chrono::Utc;
use futures::TryStreamExt;
use mongodb::{bson::doc, results::InsertOneResult, Collection, Database};

use crate::models::VoteHistory;

pub struct VoteRepository {
    collection: Collection<VoteHistory>,
}
impl VoteRepository {
    pub fn init(db: &Database) -> Self {
        let vote_history: Collection<VoteHistory> = db.collection("votes");
        VoteRepository {
            collection: vote_history,
        }
    }

    pub async fn new_vote(
        &self,
        user_id: String,
        poll_id: String,
        option_id: String,
    ) -> Result<InsertOneResult, Box<dyn std::error::Error>> {
        let current_time = Utc::now();
        let vote = VoteHistory {
            option_id,
            poll_id,
            user_id,
            created_at: current_time,
        };

        let response = self.collection.insert_one(vote).await;
        match response {
            Ok(val) => Ok(val),
            Err(e) => Err(format!("Error Adding Vote to VoteHistory , {}", e).into()),
        }
    }

    pub async fn check_vote_exists(
        &self,
        user_id: &String,
        poll_id: &String,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let filter = doc! {
            "userId" : user_id.clone(),
            "pollId" : poll_id.clone()
        };
        let result = self.collection.find_one(filter).await?;
        match result {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    pub async fn get_last_10_votes(&self, poll_id: String) -> Result<Vec<VoteHistory>, Box<dyn std::error::Error>> {
        let filter = doc! { "pollId": poll_id };
    
        let cursor = self.collection.find(filter).limit(10).await?;
        let votes: Vec<VoteHistory> = cursor.try_collect().await?;
        Ok(votes)
    }
}
