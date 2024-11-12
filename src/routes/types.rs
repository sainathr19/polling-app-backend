use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::models::{Poll, VoteHistory};

#[derive(Serialize, Deserialize)]
pub struct NewPollBody {
    pub title: String,
    pub username: String,
    pub options: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct VoteQueryParams {
    pub option_id: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClosePollBody {
    pub poll_id: String,
    pub username: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchPollQuery {
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PollUpdate {
    pub poll_id: String,
    pub votes: HashMap<String, i32>,
    pub total_votes: i32,
}

#[derive(Serialize)]
pub struct PollOverviewResponse {
    pub poll_data: Poll,
    pub last_10_votes: Vec<VoteHistory>,
}

