use crate::dbnew::{Client};

use warp::{http, Filter};
use serde::Deserialize;

// path/register_user/:user_name
async fn register_user(json_msg: Json, db_client: Client) -> Result<impl warp::Reply, warp::Rejection> {
    let db_response = db_client.register_user(&json_msg.name).await;

    match db_response {
        Err(e) => {
            Ok(warp::reply::with_status(format!("internal server error: {}", e), http::StatusCode::INTERNAL_SERVER_ERROR))
        },
        Ok(query_res) => {
            match query_res {
                Ok(_) => {
                    Ok(warp::reply::with_status(format!("user registration succeed: {}", &json_msg.name), http::StatusCode::OK))
                },
                Err(err_msg) => {
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