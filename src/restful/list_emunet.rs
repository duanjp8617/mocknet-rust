use serde::Deserialize;
use warp::Filter;

use crate::database::Client;
use crate::restful::Response;

#[derive(Deserialize)]
struct Json {
    user: String,
}

async fn list_all_emunets(
    json_msg: Json,
    db_client: Client,
) -> Result<impl warp::Reply, warp::Rejection> {
    let emunets = extract_response!(
        db_client.list_emu_net_uuid(json_msg.user).await,
        "internal_server_error",
        "operation_fail"
    );

    let resp = Response::new(true, emunets, String::new());
    Ok(warp::reply::json(&resp))
}

/// This filter accepts an HTTP request containing the name of an existing user.
/// It will retrieve all the emunet that the user currently has, place the emunet name
/// and uuid inside a JSON map, and return the result back to the client side
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
        .and(warp::path("list_emunet"))
        .and(warp::path::end())
        .and(super::parse_json_body())
        .and(db_filter)
        .and_then(list_all_emunets)
}
