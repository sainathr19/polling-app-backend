use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::{self, Sender, Receiver};
use std::sync::Arc;
use parking_lot::RwLock;
use crate::models::{Poll, VoteHistory};

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct PollUpdate{
    pub poll_data : Poll,
    pub last_10_votes : Vec<VoteHistory>
}
pub struct PollState {
    channels: Arc<RwLock<HashMap<String, Sender<PollUpdate>>>>
}

impl PollState {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new()))
        }
    }

    pub fn subscribe(&self, poll_id: &str) -> Receiver<PollUpdate> {
        let mut channels = self.channels.write();
        
        match channels.get(poll_id) {
            Some(sender) => sender.subscribe(),
            None => {
                let (sender, receiver) = broadcast::channel(32);
                channels.insert(poll_id.to_string(), sender);
                receiver
            }
        }
    }

    pub fn publish(&self, poll_id: &str, update: Poll , votes : Vec<VoteHistory>) {
        if let Some(sender) = self.channels.read().get(poll_id) {
            let poll_update = PollUpdate{
                poll_data : update,
                last_10_votes : votes
            };
            let _ = sender.send(poll_update);
        }
    }
}