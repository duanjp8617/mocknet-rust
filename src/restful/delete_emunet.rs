// Delete the emunet from the database if the emunet is in init state (i.e., it has no allocated vertexes)
// Rellocate the server resources occupied by this emunet.

use serde::Deserialize;
use warp::Filter;

use crate::database::Client;
use crate::restful::Response;

#[derive(Deserialize)]
struct Json {
    emunet_uuid: uuid::Uuid,
}

async fn delete_emunet(
    json_msg: Json,
    db_client: Client,
) -> Result<impl warp::Reply, warp::Rejection> {
    let emunet = extract_response!(
        db_client.get_emu_net(json_msg.emunet_uuid).await,
        "internal_server_error",
        "operation_fail"
    );

    extract_response!(
        db_client.delete_emunet(emunet).await,
        "internal_server_error",
        "operation_fail"
    );

    let resp = Response::new(true, (), String::new());
    Ok(warp::reply::json(&resp))
}

/// This filter imiplements the functionality to create a new emunet.
/// Note: this filter only allocate a new slot in the database to store the basic
/// information about the emunet, the actual work of creating new network nodes
/// is handled by init_emunet.rs.
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
        .and(warp::path("delete_emunet"))
        .and(warp::path::end())
        .and(super::parse_json_body())
        .and(db_filter)
        .and_then(delete_emunet)
}
