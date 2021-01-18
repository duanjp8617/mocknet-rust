use serde::{Deserialize, Serialize};
use warp::{http, Filter};

use crate::database::Client;
use crate::restful::Response;

#[derive(Deserialize)]
struct Json {
    name: String,
}

#[derive(Serialize)]
struct ResponseData {
    status: String,
    user_name: String,
}

async fn register_user(
    json_msg: Json,
    db_client: Client,
) -> Result<impl warp::Reply, warp::Rejection> {
    let _ = extract_response!(
        db_client.register_user(&json_msg.name).await,
        "internal_server_error",
        "operation_fail"
    );

    let resp_data = ResponseData {
        status: "OK".to_string(),
        user_name: json_msg.name,
    };
    let resp = Response::new(true, resp_data, String::new());

    Ok(warp::reply::with_status(
        serde_json::to_string(&resp).unwrap(),
        http::StatusCode::OK,
    ))
}

/// This filter handles an HTTP request containing a new user name.
/// It will register the user in the database and report to the sending-side
/// whether the registration succeeds.
pub fn build_filter(
    db_client: Client,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone + Send + Sync + 'static
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
