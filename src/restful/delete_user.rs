// delete the user if the user has no emunets.
use serde::Deserialize;
use warp::Filter;

use crate::database::Client;
use crate::restful::Response;

#[derive(Deserialize)]
struct Json {
    name: String,
}

async fn delete_user(
    json_msg: Json,
    db_client: Client,
) -> Result<impl warp::Reply, warp::Rejection> {
    let _ = extract_response!(
        db_client.delete_user(&json_msg.name).await,
        "internal_server_error",
        "operation_fail"
    );

    let resp = Response::new(true, json_msg.name, String::new());

    Ok(warp::reply::json(&resp))
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
        .and(warp::path("delete_user"))
        .and(warp::path::end())
        .and(super::parse_json_body())
        .and(db_filter)
        .and_then(delete_user)
}
