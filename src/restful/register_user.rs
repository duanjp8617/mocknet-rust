use crate::database::{Client, QueryOk, QueryFail};

use warp::{http, Filter};
use serde::Deserialize;

// curl --location --request POST 'localhost:3030/v1/register_user' \
// --header 'Content-Type: application/json' \
// --header 'Content-Type: text/plain' \
// --data-raw '{
//     "name": "fuck"
// }'

// path/register_user/:user_name
async fn register_user(user_name: String, db_client: Client) -> Result<impl warp::Reply, warp::Rejection> {
    let db_response = db_client.register_user(&user_name).await;

    match db_response {
        Err(e) => {
            Ok(warp::reply::with_status(format!("internal server error: {}", e), http::StatusCode::INTERNAL_SERVER_ERROR))
        },
        Ok(query_res) => {
            match query_res {
                QueryOk(_) => {
                    Ok(warp::reply::with_status(format!("user registration succeed: {}", user_name), http::StatusCode::OK))
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
    name: String,
}

fn parse_json_body() -> impl Filter<Extract = (String,), Error = warp::Rejection> + Clone {
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    warp::body::content_length_limit(1024 * 16)
        .and(warp::body::json())
        .map(|req_body: Json|{req_body.name})
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
        .and(parse_json_body())
        .and(db_filter)
        .and_then(register_user)
}