use core::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug,Clone)]
#[serde(rename_all = "camelCase")]
pub struct PollOption {
    pub option_id: String,
    pub option_text: String,
    pub votes: u64,
}

#[derive(Serialize, Deserialize, Debug,Clone)]
pub enum PollStatus {
    OPEN,
    CLOSED,
}
impl fmt::Display for PollStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PollStatus::OPEN => "OPEN",
                PollStatus::CLOSED => "CLOSED",
            }
        )
    }
}

#[derive(Serialize, Deserialize, Debug,Clone)]
#[serde(rename_all = "camelCase")]
pub struct Poll {
    pub title: String,
    pub creator_id: String,
    pub poll_id: String,
    pub options: Vec<PollOption>,
    pub status: PollStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug,Clone)]
#[serde(rename_all = "camelCase")]
pub struct VoteHistory {
    pub option_id: String,
    pub poll_id: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize,Deserialize,Debug)]
pub struct UserRegistrationState {
    pub username: String,
    pub user_unique_id: String,
    pub reg_state: serde_json::Value,
}

#[derive(Serialize,Deserialize,Debug)]
pub struct UserAuthenticationState {
    pub username: String,
    pub auth_state: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone,Debug)]
pub struct User {
    pub username: String,
    pub sk : serde_json::Value
}
