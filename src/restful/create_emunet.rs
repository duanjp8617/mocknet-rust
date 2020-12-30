use warp::{http, Filter};
use serde::{Serialize, Deserialize};

use crate::database::{Client};

#[derive(Deserialize)]
struct Json {
    user: String,
    emunet: String,
    capacity: u32,
}

#[derive(Serialize)]
struct Response {
    emunet_uuid: uuid::Uuid,
}

async fn create_emunet(json_msg: Json, db_client: Client) -> Result<impl warp::Reply, warp::Rejection> {
    let emunet_uuid = extract_response!(
        db_client.create_emu_net(json_msg.user, json_msg.emunet, json_msg.capacity).await,
        "internal_server_error",
        "operation_fail"
    ); 

    let resp = Response {emunet_uuid};

    Ok(warp::reply::with_status(serde_json::to_string(&resp).unwrap(), http::StatusCode::OK))
}

/// This filter imiplements the functionality to create a new emunet.
/// Note: this filter only allocate a new slot in the database to store the basic
/// information about the emunet, the actual work of creating new network nodes
/// is handled by init_emunet.rs.
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
        .and(super::parse_json_body())
        .and(db_filter)
        .and_then(create_emunet)
}