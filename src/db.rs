pub mod poll_repo;
pub mod user_repo;
pub mod vote_repo;
pub mod reg_state_repo;
pub mod auth_state_repo;
use std::{collections::HashMap, env, sync::Arc};

use auth_state_repo::AuthStateRepository;
use dotenv::dotenv;
use futures_util::lock::Mutex;
use mongodb::Client;
use poll_repo::PollRepository;
use reg_state_repo::RegStateRepository;
use tokio::sync::{broadcast, mpsc};
use user_repo::UserRepository;
use vote_repo::VoteRepository;

use crate::models::Poll;
pub struct MongoDB {
    pub poll_collection: PollRepository,
    pub vote_collection: VoteRepository,
    pub reg_state_collection : RegStateRepository,
    pub auth_state_collection : AuthStateRepository,
    pub user_collection : UserRepository,
    pub sender: Arc<Mutex<std::collections::HashMap<String, mpsc::Sender<Poll>>>>,}

impl MongoDB {
    pub async fn init() -> Self {
        dotenv().ok();
        let mongo_uri = env::var("DATABASE_URL").expect("DATABASE_URL must be set !");
        let client = Client::with_uri_str(mongo_uri)
            .await
            .expect("Error Connecting with Database");
        println!("Connected to MongoDB");
        let database = client.database("poll-app");
        let poll_collection = PollRepository::init(&database);
        let vote_collection = VoteRepository::init(&database);
        let user_collection = UserRepository::init(&database);
        let reg_state_collection = RegStateRepository::init(&database);
        let auth_state_collection = AuthStateRepository::init(&database);
        let sender = Arc::new(Mutex::new(HashMap::new())) ;  
        MongoDB {
            poll_collection,
            vote_collection,
            reg_state_collection,
            auth_state_collection,
            user_collection,
            sender
        }
    }
}
