pub mod poll_repo;
pub mod user_repo;
pub mod vote_repo;
pub mod reg_state_repo;
pub mod auth_state_repo;
use std::env;
use auth_state_repo::AuthStateRepository;
use dotenv::dotenv;
use mongodb::Client;
use poll_repo::PollRepository;
use reg_state_repo::RegStateRepository;
use user_repo::UserRepository;
use vote_repo::VoteRepository;

pub struct MongoDB {
    pub poll_collection: PollRepository,
    pub vote_collection: VoteRepository,
    pub reg_state_collection : RegStateRepository,
    pub auth_state_collection : AuthStateRepository,
    pub user_collection : UserRepository,
}

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
        MongoDB {
            poll_collection,
            vote_collection,
            reg_state_collection,
            auth_state_collection,
            user_collection,
        }
    }
}
