use crate::dbnew::{Client};

use warp::{http, Filter};
use serde::{Serialize, Deserialize};

#[derive(Deserialize)]
struct Json {
    name: String,
}

#[derive(Serialize)] 
struct Response {
    status: String,
    user_name: String,
}

async fn register_user(json_msg: Json, db_client: Client) -> Result<impl warp::Reply,  warp::Rejection> {
    let _ = extract_response!(
        db_client.register_user(&json_msg.name).await,
        "internal server error",
        "operation fail"
    ); 
    
    let resp = Response {
        status: "OK".to_string(),
        user_name: json_msg.name
    };
    Ok(warp::reply::with_status(serde_json::to_string(&resp).unwrap(), http::StatusCode::OK))
}

pub fn build_filter(db_client: Client) 
-> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone + Send + Sync + 'static
{
    let db_filter = warp::any().map(move || {
        let clone = db_client.clone();
        clone
    });
    warp::post()
        .and(warp::path("v1"))
        .and(warp::path("register_user"))
        .and(warp::path::end())
        .and(super::parse_json_body())
        .and(db_filter)
        .and_then(register_user)
}