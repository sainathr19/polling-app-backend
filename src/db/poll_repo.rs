use crate::helpers::time_formatter::get_current_time_bson;
use crate::models::{Poll, PollOption, PollStatus};
use chrono::Utc;
use futures_util::TryStreamExt;
use mongodb::bson::doc;
use mongodb::bson::to_document;
use mongodb::bson::Bson;
use mongodb::bson::Document;
use mongodb::results::DeleteResult;
use mongodb::results::InsertOneResult;
use mongodb::results::UpdateResult;
use mongodb::Collection;
use mongodb::Database;
pub struct PollRepository {
    collection: Collection<Poll>,
}

impl PollRepository {
    pub fn init(db: &Database) -> Self {
        let poll_collection: Collection<Poll> = db.collection("polls");
        PollRepository {
            collection: poll_collection,
        }
    }

    pub async fn create_one(
        &self,
        title: String,
        creator_id: String,
        options: Vec<PollOption>,
        poll_id: String,
        status: PollStatus,
    ) -> Result<InsertOneResult, Box<dyn std::error::Error>> {
        let current_time = Utc::now();
        let new_poll = Poll {
            title,
            creator_id,
            poll_id: poll_id.clone(),
            options,
            status,
            created_at: current_time,
            updated_at: current_time,
        };
        let response = self
            .collection
            .insert_one(new_poll)
            .await
            .map_err(|e| format!("Error Creating Poll : {:?}", e))?;
        println!("Poll creation SuccessFull With id : {}", poll_id);
        Ok(response)
    }

    pub async fn find_by_poll_id(
        &self,
        poll_id: String,
    ) -> Result<Option<Poll>, Box<dyn std::error::Error>> {
        let filter = doc! {
            "pollId": poll_id.clone()
        };
        let response: Option<Poll> = self.collection.find_one(filter).await?;
        Ok(response)
    }

    pub async fn fetch_polls(
        &self,
        user_id: Option<String>,
    ) -> Result<Vec<Poll>, Box<dyn std::error::Error>> {
        let filter = match user_id {
            Some(val) => doc! {
                "userId" : val.clone()
            },
            None => doc! {},
        };
        let cursor = self.collection.find(filter).await?;
        let results: Vec<Poll> = cursor.try_collect().await?;
        Ok(results)
    }

    pub async fn add_vote(
        &self,
        poll_id: String,
        option_id: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let filter = doc! {
            "pollId" : poll_id.clone(),
            "options.optionId" : option_id
        };
        let update = doc! {
            "$inc": {
                format!("options.$.votes"): 1
            },
            "$set":{
                "updatedAt" : get_current_time_bson()
            }
        };
        let response = self.collection.update_one(filter, update).await;
        match response {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Error adding vote for pollId {} , {}", poll_id, e).into()),
        }
    }

    pub async fn get_poll_creator(
        &self,
        poll_id: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let filter = doc! {
            "pollId" : poll_id.clone()
        };

        let response = self.collection.find_one(filter).await?;

        match response {
            Some(value) => Ok(value.creator_id),
            None => return Err(format!("No Poll found with the PollId : {}", poll_id).into()),
        }
    }
    pub async fn reset_poll_stats(
        &self,
        poll_id: String,
    ) -> Result<UpdateResult, Box<dyn std::error::Error>> {
        let filter = doc! {
            "pollId": &poll_id
        };

        let poll_doc = self.collection.find_one(filter.clone()).await?;
        let poll_doc = match poll_doc {
            Some(doc) => doc,
            None => return Err("Poll not found".into()),
        };

        let poll_options: Vec<PollOption> = poll_doc.options;
        let mut new_options: Vec<Document> = Vec::new();
        for option in poll_options {
            let new_option = PollOption {
                option_id: option.option_id.clone(),
                option_text: option.option_text.clone(),
                votes: 0,
            };
            new_options.push(to_document(&new_option)?);
        }

        let update = doc! {
            "$set": {
                "options":  new_options,
                "updatedAt" :get_current_time_bson()
            }
        };

        let response = self.collection.update_one(filter, update).await?;
        Ok(response)
    }

    pub async fn delete_poll_by_id(
        &self,
        poll_id: String,
    ) -> Result<DeleteResult, Box<dyn std::error::Error>> {
        let filter = doc! {
            "pollId": poll_id.clone()
        };

        let response = self.collection.delete_one(filter).await?;
        Ok(response)
    }

    pub async fn close_poll_by_id(
        &self,
        poll_id: String,
    ) -> Result<UpdateResult, Box<dyn std::error::Error>> {
        let filter = doc! {
            "pollId": poll_id.clone()
        };
        let update = doc! {
            "$set": {
                "status": Bson::String(PollStatus::CLOSED.to_string()),
                "updatedAt" : get_current_time_bson()
            }
        };

        let response = self.collection.update_one(filter, update).await?;
        Ok(response)
    }
}
