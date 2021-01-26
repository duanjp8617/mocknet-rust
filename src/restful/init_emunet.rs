use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tokio::time;
use warp::Filter;

use crate::algo::in_memory_graph::InMemoryGraph;
use crate::algo::Partition;
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

// given a client-side edge id, return the database-side edge uuid and the
// corresponding vertex to insert this directed edge
// note: the caller should ensure that this function never panics the program
fn insert_edge_helper<'a>(
    e_id: (u64, u64),
    id_map: &HashMap<u64, uuid::Uuid>,
    vertex_map: &'a mut HashMap<u64, Vertex>,
) -> ((uuid::Uuid, uuid::Uuid), &'a mut Vertex) {
    let e_uuid = (
        id_map.get(&e_id.0).unwrap().clone(),
        id_map.get(&e_id.1).unwrap().clone(),
    );
    let vertex_mut = vertex_map.get_mut(&e_id.0).unwrap();
    (e_uuid, vertex_mut)
}

// helper function to update error state on the emunet object
async fn emunet_error(client: Client, mut emunet: EmuNet, err: EmuNetError) {
    emunet.error(err);
    // store the error state in the database, panic the server program on failure
    let res = client
        .set_emu_net(emunet)
        .await
        .expect("this should not happen");
    if res.is_err() {
        panic!("this should never happen");
    }
}

// the actual work is done in a background task
async fn background_task(
    client: Client,
    mut emunet: EmuNet,
    network_graph: InMemoryGraph<u64, VertexInfo, EdgeInfo>,
) {
    // record the network size
    let size = network_graph.size() as u32;

    // do the partition
    let res = network_graph.partition(emunet.servers_mut());
    if res.is_err() {
        // set the state of the emunet to fail
        let err = EmuNetError::PartitionFail(format!("{}", res.map(|_| { () }).unwrap_err()));
        emunet_error(client, emunet, err).await;
        return;
    }
    let assignment = res.unwrap();

    // create a vertex-id-to-uuid map
    // prepare the EdgeInfo list, which will be used later
    let (vertex_infos, edge_infos) = network_graph.into();
    let id_map: HashMap<u64, uuid::Uuid> =
        vertex_infos.iter().fold(HashMap::new(), |mut map, vi| {
            if map
                .insert(vi.id(), indradb::util::generate_uuid_v1())
                .is_some()
            {
                panic!("fatal".to_string());
            }
            map
        });

    // build up a map from the client-side id to the vertex
    let mut vertexes_map: HashMap<u64, Vertex> =
        vertex_infos
            .into_iter()
            .fold(HashMap::new(), |mut map, vi| {
                let client_id = vi.id();
                let v = Vertex::new(
                    vi,
                    id_map.get(&client_id).unwrap().clone(),
                    assignment.get(&client_id).unwrap().clone(),
                );
                if map.insert(client_id, v).is_some() {
                    panic!("fatal".to_string());
                }
                map
            });
    // insert the edges into the vertexes
    let _: Vec<_> = edge_infos
        .into_iter()
        .map(|ei| {
            // insert the edge with forward direction
            let e_id = ei.edge_id();
            let (e_uuid, vertex_mut) = insert_edge_helper(e_id, &id_map, &mut vertexes_map);
            let edge = Edge::new(e_uuid, ei.description());
            vertex_mut.add_edge(edge).unwrap();

            // insert the edge with reverse direction
            let e_id = ei.reverse_edge_id();
            let (e_uuid, vertex_mut) = insert_edge_helper(e_id, &id_map, &mut vertexes_map);
            let edge = Edge::new(e_uuid, ei.description());
            vertex_mut.add_edge(edge).unwrap();
        })
        .collect();

    // create the vertexes in the database
    let res = client
        .bulk_create_vertexes(
            vertexes_map.values().map(|v| v.uuid()),
            emunet.vertex_type(),
        )
        .await;
    match res {
        Ok(_) => {}
        Err(err) => {
            // set the state of the emunet to fail
            let err = EmuNetError::DatabaseFail(format!("{:?}", err));
            emunet_error(client, emunet, err).await;
            return;
        }
    };

    // set the vertex properties
    let res = client
        .bulk_set_vertex_properties(
            vertexes_map
                .values()
                .map(|v| (v.uuid(), serde_json::to_value(v.clone()).unwrap())),
        )
        .await;
    match res {
        Ok(_) => {}
        Err(err) => {
            // set the state of the emunet to fail
            let err = EmuNetError::DatabaseFail(format!("{:?}", err));
            emunet_error(client, emunet, err).await;
            return;
        }
    };

    // emulate the background task of launching containers and creating connections
    time::delay_for(time::Duration::new(5, 0)).await;
    // potentially perform an update on the vertexes

    // set the state of the emunet to normal
    emunet.normal();
    // store the vertex mappings in to the emunet
    id_map.into_iter().fold(&mut emunet, |emunet, mapping| {
        emunet.add_vertex(mapping.0, mapping.1);
        emunet
    });
    // reserve the capacity for the emunet
    emunet.reserve_capacity(size);

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
        "internal_server_error",
        "operation_fail"
    );
    if !emunet.is_uninit() {
        // emunet can only be initialized once
        return Ok(warp::reply::json(&Response::<()>::new(
            false,
            (),
            "operation_fail: EmuNet can only be initialized once".to_string(),
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
    if network_graph.size() > emunet.capacity() as usize {
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
    tokio::spawn(background_task(db_client, emunet, network_graph));

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

/// This filter initializes the emunet by creating the vertexes and edges of the emulation network.
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
        .and(warp::path("init_emunet"))
        .and(warp::path::end())
        .and(super::parse_json_body())
        .and(db_filter)
        .and_then(init_emunet)
}
