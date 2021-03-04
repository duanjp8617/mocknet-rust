use serde::{Deserialize, Serialize};
use warp::Filter;

use crate::algo::in_memory_graph::InMemoryGraph;
use crate::database::Client;
use crate::emunet::net::*;
use crate::restful::Response;


// format of the incoming json message
#[derive(Deserialize)]
struct Json {
    emunet_uuid: uuid::Uuid, // uuid of the emunet object on the database
    devs: Vec<VertexInfo>,   // a list of vertexes to be created
    links: Vec<EdgeInfo>,    // a list of edges to be created
}

// the actual work is done in a background task
async fn update_background_task(
    client: Client,
    emunet: EmuNet,
    network_graph: InMemoryGraph<u64, VertexInfo, EdgeInfo>,
) {
    let emunet_uuid = emunet.uuid().clone();
    super::destruct_emunet::destruct_background_task(client.clone(), emunet, false).await;
    
    let res = client.get_emu_net(emunet_uuid).await;
    if res.is_err() {
        return;
    }
    let res = res.unwrap();
    if res.is_err() {
        return;
    }
    let emunet = res.unwrap();
    
    super::init_emunet::init_background_task(client, emunet, network_graph).await;
}

// path/create_emunet/
async fn update_emunet(json: Json, db_client: Client) -> Result<impl warp::Reply, warp::Rejection> {
    // retrieve the emunet object from the database
    let mut emunet = extract_response!(
        db_client.get_emu_net(json.emunet_uuid.clone()).await,
        "internal_server_error",
        "operation_fail"
    );
    if !emunet.is_normal() {
        // we can only destruct an emunet that is in normal state
        return Ok(warp::reply::json(&Response::<()>::new(
            false,
            (),
            "operation_fail: we can only destruct Emunet in normal state".to_string(),
        )));
    };

    // build up the in memory graph
    let res = InMemoryGraph::from_vecs(
        json.devs.into_iter().map(|v| (v.id(), v)).collect(),
        json.links.into_iter().map(|e| (e.edge_id(), e)).collect(),
    );
    if res.is_err() {
        // InMemoryGraph<u64, VertexInfo,EdgeInfo> does not implement fmt::Debug,
        // map it to () and then extract the error message
        return Ok(warp::reply::json(&Response::<()>::new(
            false,
            (),
            format!(
                "\"invalid_input_graph\": \"{}\"",
                res.map(|_| { () }).unwrap_err()
            ),
        )));
    }
    let network_graph: InMemoryGraph<u64, VertexInfo, EdgeInfo> = res.unwrap();
    if network_graph.size() > emunet.max_capacity() as usize {
        // report error if the input network topology exceeds the capacity
        // of the emunet
        return Ok(warp::reply::json(&Response::<()>::new(
            false,
            (),
            "invalid_input_graph: input graph exceeds capacity limitation".to_string(),
        )));
    }

    // update the state of the emunet object into working
    emunet.working();
    let _ = extract_response!(
        db_client.set_emu_net(emunet.clone()).await,
        "internal_server_error",
        "fatal(this-should-never-happen)"
    );

    // do the actual initialization work in the background
    tokio::spawn(update_background_task(db_client, emunet, network_graph));

    // reply to the client
    #[derive(Serialize)]
    struct ResponseData {
        status: String,
    }
    Ok(warp::reply::json(&Response::<ResponseData>::new(
        true,
        ResponseData {
            status: "working".to_string(),
        },
        String::new(),
    )))
}

/// This filter updates the emunet by replacing the network topology into a new one.
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
        .and(warp::path("update_emunet"))
        .and(warp::path::end())
        .and(super::parse_json_body())
        .and(db_filter)
        .and_then(update_emunet)
}
