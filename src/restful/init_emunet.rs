use std::collections::HashSet;

use warp::{http, Filter};
use warp::reply::with_status;
use http::StatusCode;
use serde::Deserialize;

use crate::dbnew::{Client};
use crate::emunet::net::*;
use crate::algo::in_memory_graph::InMemoryGraph;
use crate::algo::Partition;


// curl --location --request POST 'localhost:3030/v1/init_emunet' \
// --header 'Content-Type: application/json' \
// --header 'Content-Type: text/plain' \
// --data-raw '{
//     "user": "wtf",
//     "emunet": "you",
//     "capacity": 24,
//     "devs": [],
//     "links": [],
// }'

async fn background_task(client: Client, mut emunet: EmuNet, network_graph: InMemoryGraph<u64, VertexInfo,EdgeInfo>) {
    // do the allocation
    let res = network_graph.partition(emunet.servers_mut());
    if res.is_err() {
        // set the state of the emunet to fail
        emunet.Error(EmuNetError::PartitionFail(format!("{}", res.map(|_|{()}).unwrap_err())));
        
        // store the state in the database, panic the server program on failure
        let res = client.set_emu_net(emunet).await.unwrap();
        if res.is_err() {
            panic!("this should never happen");
        }
        
        // quit the background task
        return;
    }

    // acquire the partition result
    let assignment = res.unwrap();
    // get the lists of vertex_info and edge_info
    let (vertex_infos, edge_infos) = network_graph.into();

    // build up the list of vertexes
    let vertexes: Vec<Vertex> = vertex_infos.into_iter().map(|vi| {
        let client_id = vi.id();
        Vertex::new(
            vi, 
            indradb::util::generate_uuid_v1(), 
            assignment.get(&client_id).unwrap().clone()
        )
    }).collect();



    // prepare the list of bulk insert items
    let mut bulk_vertexes: Vec<indradb::BulkInsertItem> = Vec::new();
    let _: Vec<()> = vertexes.iter().map(|v| {
        let uuid = v.uuid();
        emunet.add_vertex_assignment(v.id(), uuid.clone());
        let vertex = indradb::Vertex::with_id(uuid, indradb::Type::new("t").unwrap());
        bulk_vertexes.push(indradb::BulkInsertItem::Vertex(vertex));
    }).collect();
    // insert the bulk vertexes into the fucking database

    let mut bulk_vertex_properties: Vec<indradb::BulkInsertItem> = Vec::new();
    let _: Vec<()> = vertexes.into_iter().map(|v|{
        let uuid = v.uuid();
        let json_value = serde_json::to_value(v).unwrap();
        bulk_vertex_properties.push(indradb::BulkInsertItem::VertexProperty(uuid, "default".to_string(), json_value));
    }).collect();
    // insert the bulk vertexes into the database
    
}

// path/create_emunet/
async fn init_emunet(json: Json, db_client: Client) -> Result<impl warp::Reply, warp::Rejection> {
    // retrieve the emunet from the database
    let mut emunet = extract_response!(
        db_client.get_emu_net(json.emunet_uuid.clone()).await,
        "internal server error",
        "operation fail"
    );
    // make sure that the emunet has not been initialized
    match emunet.state() {
        EmuNetState::Uninit => {},
        _ => return Ok(with_status("operation fail: EmuNet is not initialized".to_string(), StatusCode::BAD_REQUEST)),
    };

    // build up the in memory graph
    let res = InMemoryGraph::from_vecs(
        json.devs.into_iter().map(|v|{(v.id(), v)}).collect(), 
        json.links.into_iter().map(|e|{(e.edge_id(), e)}).collect(),
    );
    if res.is_err() {
        // InMemoryGraph<u64, VertexInfo,EdgeInfo> does not implement fmt::Debug,
        // map it to () and then extract the error message
        return Ok(with_status(format!("invalid input graph: {}", res.map(|_|{()}).unwrap_err()), StatusCode::BAD_REQUEST));
    }
    // create the in-memory graph if the input graph is valid
    let network_graph: InMemoryGraph<u64, VertexInfo, EdgeInfo> = res.unwrap();
    
    // update the state of the emunet into working
    emunet.working();
    let _ = extract_response!(
        db_client.set_emu_net(emunet.clone()).await,
        "internal server error",
        "this should never happen!"
    );
    
    // do the actual initialization work in the background
    tokio::spawn(background_task(db_client, emunet, network_graph));
    
    // reply to the client
    Ok(warp::reply::with_status(format!("emunet is initializing."), http::StatusCode::CREATED))
}

#[derive(Deserialize)]
struct Json {
    emunet_uuid: uuid::Uuid,
    devs: Vec<VertexInfo>,
    links: Vec<EdgeInfo>,
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
        .and(warp::path("init_emunet"))
        .and(warp::path::end())
        .and(parse_json_body())
        .and(db_filter)
        .and_then(init_emunet)
}