use std::collections::HashMap;

use warp::{http, Filter};
use warp::reply::with_status;
use http::StatusCode;
use serde::Deserialize;
use tokio::time;

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

// format of the incoming json message
#[derive(Deserialize)]
struct Json {
    emunet_uuid: uuid::Uuid, // uuid of the emunet object on the database
    devs: Vec<VertexInfo>, // a list of vertexes to be created
    links: Vec<EdgeInfo>, // a list of edges to be created
}

// helper function to update error state on the emunet object
async fn emunet_error(client: Client, mut emunet: EmuNet, err: EmuNetError) {
    emunet.error(err);
    // store the error state in the database, panic the server program on failure
    let res = client.set_emu_net(emunet).await.expect("this should not happen");
    if res.is_err() {
        panic!("this should never happen");
    }
}

// the actual work is done in a background task
async fn background_task(client: Client, mut emunet: EmuNet, network_graph: InMemoryGraph<u64, VertexInfo,EdgeInfo>) {
    // do the allocation
    let res = network_graph.partition(emunet.servers_mut());
    if res.is_err() {
        // set the state of the emunet to fail
        let err = EmuNetError::PartitionFail(format!("{}", res.map(|_|{()}).unwrap_err()));
        emunet_error(client, emunet, err).await;
        return;
    }

    // acquire the partition result, which assigns each vertex to a server
    let assignment = res.unwrap();
    // get the lists of vertex_info and edge_info
    let (vertex_infos, edge_infos) = network_graph.into();

    // create a vertex id to uuid map
    let id_map: HashMap<u64, uuid::Uuid> = vertex_infos.iter().fold(HashMap::new(), |mut map, vi| {
        if map.insert(vi.id(), indradb::util::generate_uuid_v1()).is_some() {
            panic!("fatal".to_string());
        }
        map
    });

    // build up a map from the client-side id to the vertex
    let mut vertexes_map: HashMap<u64, Vertex> = vertex_infos.into_iter().fold(HashMap::new(), |mut map, vi| {
        let client_id = vi.id();
        let v = Vertex::new(
            vi, 
            id_map.get(&client_id).unwrap().clone(), 
            assignment.get(&client_id).unwrap().clone()
        );
        if map.insert(client_id, v).is_some() {
            panic!("fatal".to_string());
        }
        map
    });
    // insert the edges into the vertexes
    let _: Vec<_> = edge_infos.into_iter().map(|ei| {
        let e_id = ei.edge_id();
        let e_uuid = (id_map.get(&e_id.0).unwrap().clone(), id_map.get(&e_id.1).unwrap().clone());
        let vertex_mut = vertexes_map.get_mut(&e_id.0).unwrap();
        
        let edge = Edge::new(e_uuid, ei.description());

        vertex_mut.add_edge(edge).unwrap();
    }).collect();
    // convert the vertexes_map back into a list of vertexes
    let vertexes = vertexes_map.into_iter().fold(Vec::new(), |mut vec, (_, v)| {
        vec.push(v);
        vec
    });

    // create the vertexes
    let res = client.bulk_create_vertexes(vertexes.iter().map(|v|{v.uuid()}).collect(), emunet.vertex_type()).await;
    match res {
        Ok(_) => {},
        Err(err) => {
            // set the state of the emunet to fail
            let err = EmuNetError::DatabaseFail(format!("{:?}", err));
            emunet_error(client, emunet, err).await;
            return;
        }
    };

    // set the vertex properties
    let res = client.bulk_set_vertex_properties(
        vertexes.iter().map(
            |v| {
                (v.uuid(), serde_json::to_value(v.clone()).unwrap())
            }
        ).collect()
    ).await;
    match res {
        Ok(_) => {},
        Err(err) => {
            // set the state of the emunet to fail
            let err = EmuNetError::DatabaseFail(format!("{:?}", err));
            emunet_error(client, emunet, err).await;
            return;
        }
    };

    // emulate the background task of launching containers and creating connections
    time::delay_for(time::Duration::new(5,0)).await;
    // potentially perform an update on the vertexes

    // set the state of the emunet to fail
    emunet.normal();
    // store the vertex mappings in to the emunet
    id_map.into_iter().fold(&mut emunet, |emunet, mapping| {
        emunet.add_vertex(mapping.0, mapping.1);
        emunet
    });
            
    // store the state in the database, panic the server program on failure
    let res = client.set_emu_net(emunet).await.unwrap();
    if res.is_err() {
        panic!("this should never happen");
    }
}

// path/create_emunet/
async fn init_emunet(json: Json, db_client: Client) -> Result<impl warp::Reply, warp::Rejection> {
    // retrieve the emunet object from the database
    let mut emunet = extract_response!(
        db_client.get_emu_net(json.emunet_uuid.clone()).await,
        "internal server error",
        "operation fail"
    );    
    if !emunet.is_uninit() {
        // emunet can only be initialized once
        return Ok(with_status("operation fail: EmuNet can not be initialized".to_string(), StatusCode::BAD_REQUEST));
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