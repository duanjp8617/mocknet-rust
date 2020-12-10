use crate::database::{Client, QueryOk, QueryFail};

use warp::{http, Filter};
use serde::Deserialize;

use std::collections::HashSet;

// curl --location --request POST 'localhost:3030/v1/init_emunet' \
// --header 'Content-Type: application/json' \
// --header 'Content-Type: text/plain' \
// --data-raw '{
//     "user": "wtf",
//     "emunet": "you",
//     "capacity": 24,
//     "devs": [],
//     "links": [],
// }'

// path/create_emunet/
async fn init_emunet(json_value: Json, db_client: Client) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::with_status(format!("emunet is initializing."), http::StatusCode::CREATED))
}

#[derive(Deserialize)]
struct DevJson {
    digit_id: u64,
    name: String,
}

#[derive(Deserialize)]
struct LinkJson {
    src_id: u64,
    dst_id: u64,
}

#[derive(Deserialize)]
struct Json {
    user: String,
    emunet: String,
    capacity: u32,
    devs: Vec<DevJson>,
    links: Vec<LinkJson>,
}

fn parse_json_body() -> impl Filter<Extract = (Json,), Error = warp::Rejection> + Clone {
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    warp::body::content_length_limit(1024 * 16)
        .and(warp::body::json())
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
        .and(warp::path("init_emunet"))
        .and(warp::path::end())
        .and(parse_json_body())
        .and(db_filter)
        .and_then(init_emunet)
}