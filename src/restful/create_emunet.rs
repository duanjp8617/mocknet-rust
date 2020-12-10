use crate::database::{Client, QueryOk, QueryFail};

use warp::{http, Filter};
use serde::Deserialize;

// curl --location --request POST 'localhost:3030/v1/create_emunet' \
// --header 'Content-Type: application/json' \
// --header 'Content-Type: text/plain' \
// --data-raw '{
//     "user": "wtf",
//     "emunet": "you",
//     "capacity": 24
// }'

// path/create_emunet/
async fn create_emunet(json_value: Json, db_client: Client) -> Result<impl warp::Reply, warp::Rejection> {
    // unwrap ClientError
    let db_response = db_client.create_emu_net(json_value.user, json_value.emunet, json_value.capacity).await;

    match db_response {
        Err(e) => {
            Ok(warp::reply::with_status(format!("internal server error: {}", e), http::StatusCode::INTERNAL_SERVER_ERROR))
        },
        Ok(query_res) => {
            match query_res {
                QueryOk(uuid) => {
                    Ok(warp::reply::with_status(format!("emunet_uuid: {}", uuid), http::StatusCode::OK))
                },
                QueryFail(err_msg) => {
                    Ok(warp::reply::with_status(format!("operation fail: {}", err_msg), http::StatusCode::BAD_REQUEST))
                }
            }
        }
    }
}

#[derive(Deserialize)]
struct Json {
    user: String,
    emunet: String,
    capacity: u32,
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
        .and(warp::path("create_emunet"))
        .and(warp::path::end())
        .and(parse_json_body())
        .and(db_filter)
        .and_then(create_emunet)
}