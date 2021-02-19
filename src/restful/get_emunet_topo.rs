use serde::{Deserialize, Serialize};
use warp::Filter;

use crate::database::Client;
use crate::emunet::net;
use crate::restful::Response;

#[derive(Deserialize)]
struct Json {
    emunet_uuid: uuid::Uuid,
}

#[derive(Serialize)]
struct ResponseData {
    vertex_infos: Vec<net::VertexInfo>,
    edge_infos: Vec<net::EdgeInfo>,
}

async fn get_emunet(
    json_msg: Json,
    db_client: Client,
) -> Result<impl warp::Reply, warp::Rejection> {
    let emunet = extract_response!(
        db_client.get_emu_net(json_msg.emunet_uuid).await,
        "internal_server_error",
        "operation_fail"
    );

    let (vertex_infos, edge_infos) = extract_response!(
        db_client.get_emu_net_infos(&emunet).await,
        "internal_server_error",
        "operation_fail"
    );

    let resp_data = ResponseData {
        vertex_infos,
        edge_infos,
    };
    let resp = Response::new(true, resp_data, String::new());

    Ok(warp::reply::json(&resp))
}

/// This filter retrieves the topology of the emunet from the database.
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
        .and(warp::path("get_emunet_topo"))
        .and(warp::path::end())
        .and(super::parse_json_body())
        .and(db_filter)
        .and_then(get_emunet)
}
