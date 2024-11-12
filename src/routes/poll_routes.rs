use futures::StreamExt;
use crate::db::MongoDB;
use crate::helpers::poll_state::PollState;
use crate::models::PollOption;
use crate::models::PollStatus;
use crate::routes::types::{FetchPollQuery, NewPollBody,PollOverviewResponse};
use actix_web::web::Bytes;
use tokio_stream::wrappers::BroadcastStream;
use actix_web::get;
use actix_web::{
    post,
    web::{self, Data},
    HttpResponse, Responder,
};
use nanoid::nanoid;
use super::types::VoteQueryParams;

#[post("/new")]
async fn create_new_poll(
    mongo_db: Data<MongoDB>,
    poll_data: web::Json<NewPollBody>,
) -> impl Responder {
    let poll_id = nanoid!(10);
    let title = poll_data.title.clone();
    let creator_id = poll_data.username.clone();
    let status = PollStatus::OPEN;

    let options: Vec<PollOption> = poll_data
        .options
        .iter()
        .map(|option_text| PollOption {
            option_id: nanoid!(10),
            option_text: option_text.clone(),
            votes: 0,
        })
        .collect();
    let response = mongo_db
        .poll_collection
        .create_one(title, creator_id, options, poll_id, status)
        .await;
    match response {
        Ok(_) => HttpResponse::Ok().body("Poll creation SuccessFul".to_string()),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("/all")]
async fn fetch_polls(mongo_db: Data<MongoDB>, query: web::Query<FetchPollQuery>) -> impl Responder {
    let user_id = query.into_inner().user_id;
    let response = mongo_db.poll_collection.fetch_polls(user_id).await;
    let poll_data = match response {
        Ok(polls) => {
            if polls.is_empty() {
                return HttpResponse::NotFound().body("No Polls Found".to_string());
            }
            polls
        }
        Err(err) => {
            return HttpResponse::InternalServerError()
                .body(format!("Internal Server Error : {err}").to_string())
        }
    };
    HttpResponse::Ok().json(poll_data)
}

#[get("/{poll_id}/search")]
async fn fetch_poll_data(mongo_db: Data<MongoDB>, poll_id: web::Path<String>) -> impl Responder {
    let response = mongo_db
        .poll_collection
        .find_by_poll_id(poll_id.to_string())
        .await;
    match response {
        Ok(result) => match result {
            Some(val) => HttpResponse::Ok().json(val),
            None => HttpResponse::NotFound()
                .body(format!("No Poll Found with Id : {}", &poll_id).to_string()),
        },
        Err(error) => HttpResponse::InternalServerError().body(error.to_string()),
    }
}

#[post("/{poll_id}/vote")]
async fn add_new_vote(
    mongo_db: Data<MongoDB>,
    poll_id: web::Path<String>,
    query: web::Query<VoteQueryParams>,
    poll_state: Data<PollState>,
) -> impl Responder {
    let poll_id = poll_id.into_inner();
    let option_id = query.option_id.clone();
    let user_id = "sainathr19".to_string();

    // Check if the user has already voted
    // let check_user_vote = mongo_db
    //     .vote_collection
    //     .check_vote_exists(&user_id, &poll_id)
    //     .await;
    // let is_voted_user = match check_user_vote {
    //     Ok(val) => val,
    //     Err(_) => false,
    // };

    // if is_voted_user {
    //     return HttpResponse::Forbidden().body("User already voted");
    // }

    let _ = mongo_db
        .vote_collection
        .new_vote(user_id.clone(), poll_id.clone(), option_id.clone())
        .await;

    let response = mongo_db
        .poll_collection
        .add_vote(poll_id.clone(), option_id)
        .await;

    let _ = match mongo_db.poll_collection.find_by_poll_id(poll_id.clone()).await {
        Ok(Some(poll)) => {
            let last_votes = mongo_db.vote_collection.get_last_10_votes(poll_id.clone()).await.unwrap();
            poll_state.publish(&poll_id, poll,last_votes);
        },
        Ok(None) => {
            return HttpResponse::InternalServerError().json("Unknown error occurred");
        }
        Err(err) => {
            println!("{:?}", err);
            return HttpResponse::InternalServerError().json("Error fetching latest poll data");
        }
    };

    match response {
        Ok(_) => {
            HttpResponse::Ok().body(format!("Vote casting successful for PollId: {}", poll_id))
        }
        Err(_) => HttpResponse::InternalServerError().body("Error casting vote"),
    }
}


#[post("/{poll_id}/close")]
async fn close_poll(mongo_db: Data<MongoDB>, poll_id : web::Path<String>) -> impl Responder {
    let user_id = String::from("sainathr19");
    let poll_id = poll_id.into_inner();
    let poll_creator = match mongo_db
        .poll_collection
        .get_poll_creator(poll_id.clone())
        .await
    {
        Ok(creator) => creator,
        Err(error) => {
            return HttpResponse::InternalServerError()
                .body(format!("Error fetching poll creator: {}", error))
        }
    };

    if poll_creator != user_id {
        return HttpResponse::Forbidden().body("Only the poll creator can close the poll.");
    }

    let response = mongo_db
        .poll_collection
        .close_poll_by_id(poll_id.clone())
        .await;
    match response {
        Ok(_) => HttpResponse::Ok()
            .body(format!("Poll with id {} Closed SuccessFully", poll_id).to_string()),
        Err(error) => HttpResponse::InternalServerError()
            .body(format!("Error Closing Poll : {}", error).to_string()),
    }
}

#[post("/{poll_id}/reset")]
pub async fn reset_poll(mongo_db: Data<MongoDB>, poll_id: web::Path<String>) -> impl Responder {
    let poll_id = poll_id.into_inner();
    let user_id = String::from("sainathr19");
    let poll_creator = match mongo_db
        .poll_collection
        .get_poll_creator(poll_id.clone())
        .await
    {
        Ok(creator) => creator,
        Err(error) => {
            return HttpResponse::InternalServerError()
                .body(format!("Error fetching poll creator: {}", error))
        }
    };

    if poll_creator != user_id {
        return HttpResponse::Forbidden().body("Only the poll creator can reset the poll.");
    }

    let response = mongo_db
        .poll_collection
        .reset_poll_stats(poll_id.clone())
        .await;
    match response {
        Ok(_) => HttpResponse::Ok()
            .body(format!("Poll reset SuccessFull for pollId : {}", poll_id).to_string()),
        Err(error) => HttpResponse::InternalServerError()
            .body(format!("Poll reset Failed for pollId : {} , {}", poll_id, error).to_string()),
    }
}

#[get("/{poll_id}/delete")]
pub async fn delete_poll(mongo_db: Data<MongoDB>, poll_id: web::Path<String>) -> impl Responder {
    let poll_id = poll_id.into_inner();
    let user_id = String::from("sainathr19");
    let poll_creator = match mongo_db
        .poll_collection
        .get_poll_creator(poll_id.clone())
        .await
    {
        Ok(creator) => creator,
        Err(error) => {
            return HttpResponse::InternalServerError()
                .body(format!("Error fetching poll creator: {}", error))
        }
    };

    if poll_creator != user_id {
        return HttpResponse::Forbidden().body("Only the poll creator can Delete the poll.");
    }

    let response = mongo_db
        .poll_collection
        .delete_poll_by_id(poll_id.clone())
        .await;
    match response {
        Ok(_) => HttpResponse::Ok()
            .body(format!("Poll with Id {} deleted SuccessFully", poll_id).to_string()),
        Err(err) => {
            println!("{:?}", err);
            HttpResponse::InternalServerError()
                .body(format!("Failed to Delete Poll with Id : {}", poll_id).to_string())
        }
    }
}



#[get("/{poll_id}/live")]
async fn live_poll_updates(
    poll_id: web::Path<String>,
    state: Data<PollState>,
) -> impl Responder {
    let receiver = state.subscribe(&poll_id);
    let stream = BroadcastStream::new(receiver)
        .map(|msg| {
            match msg {
                Ok(update) => {
                    let data = serde_json::to_string(&update).unwrap_or_default();
                    Ok::<_, actix_web::Error>(Bytes::from(format!("data: {}\n\n", data)))
                }
                Err(_) => Ok(Bytes::from("\n")),
            }
        });

    HttpResponse::Ok()
        .insert_header(("Content-Type", "text/event-stream"))
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("Connection", "keep-alive"))
        .streaming(stream)
}

#[get("/{poll_id}/overview")]
async fn get_poll_overview(
    mongo_db: Data<MongoDB>,
    poll_id: web::Path<String>,
) -> impl Responder {
    let poll_id = poll_id.into_inner();

    let poll_data = match mongo_db.poll_collection.find_by_poll_id(poll_id.clone()).await {
        Ok(Some(poll)) => poll,
        Ok(None) => {
            return HttpResponse::NotFound().json("Poll not found");
        }
        Err(err) => {
            println!("Error fetching poll data: {:?}", err);
            return HttpResponse::InternalServerError().json("Error fetching poll data");
        }
    };

    let last_10_votes = match mongo_db.vote_collection.get_last_10_votes(poll_id.clone()).await {
        Ok(votes) => votes,
        Err(err) => {
            println!("Error fetching last 10 votes: {:?}", err);
            return HttpResponse::InternalServerError().json("Error fetching last 10 votes");
        }
    };
    let response = PollOverviewResponse {
        poll_data,
        last_10_votes,
    };

    HttpResponse::Ok().json(response)
}


pub fn init(config: &mut web::ServiceConfig) {
    config
        .service(create_new_poll)
        .service(fetch_poll_data)
        .service(add_new_vote)
        .service(close_poll)
        .service(reset_poll)
        .service(fetch_polls)
        .service(delete_poll)
        .service(get_poll_overview)
        .service(live_poll_updates);
}
