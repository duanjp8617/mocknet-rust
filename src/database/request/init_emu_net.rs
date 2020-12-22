use std::mem::replace;

use serde::{Deserialize, Serialize};

use crate::database::message::{Response, ResponseFuture, DatabaseMessage};
use crate::database::errors::BackendError;
use crate::database::backend::IndradbClientBackend;
use crate::emunet::net::{self, EmuNetState};
use crate::algo::in_memory_graph::InMemoryGraph;
use crate::algo::Partition;

#[derive(Deserialize, Serialize)]
pub struct VertexInfo {
    client_id: u64, // client side vertex id in the form of u64
    description: String, // a description string to hold the place
}

#[derive(Deserialize, Serialize)]
pub struct EdgeInfo {
    client_id: (u64, u64), // client side edge id in the form of (u64, u64)
    description: String, // a description string to hold the place
}

#[derive(Deserialize, Serialize)]
pub struct Vertex {
    client_info: VertexInfo,
    uuid: uuid::Uuid,
    server_uuid: uuid::Uuid, // which server this vertex is launched on
}

#[derive(Deserialize, Serialize)]
pub struct Edge {
    client_info: EdgeInfo,
    edge_key: (uuid::Uuid, uuid::Uuid), // out-going vertex -> incoming vertex
    description: String, // a description string to hold the place
}


pub struct InitEmuNet {
    // the Uuid of the emunet node
    emunet_uuid: uuid::Uuid,
    // a list of vertexes stored as a JSON value
    vertexes_json: serde_json::Value,
    // a list of edges stored as a JSON value
    edges_json: serde_json::Value,
}

impl DatabaseMessage<Response, BackendError> for InitEmuNet {
    fn execute<'a>(&mut self, backend: &'a IndradbClientBackend) -> ResponseFuture<'a> {        
        let msg = replace(self, InitEmuNet {
            emunet_uuid: indradb::util::generate_uuid_v1(),
            vertexes_json: serde_json::to_value(()).unwrap(),
            edges_json: serde_json::to_value(()).unwrap(),
        });
        
        Box::pin(async move {
            let msg = msg;

            // read EmuNet from the database
            let res = backend.get_vertex_json_value(msg.emunet_uuid, "default").await?;
            let mut emunet: net::EmuNet = match res {
                Some(jv) => serde_json::from_value(jv).unwrap(),
                None => return fail!(InitEmuNet, "emunet not exist".to_string()),                
            };
            // make sure that the emunet has not been initialized
            if let EmuNetState::Uninit = emunet.state() {
                // This is the desired state, do nothing
            }
            else {
                return fail!(InitEmuNet, "the emunet has already been initialized".to_string());
            };

            // deserialize the vertexes from the json message
            let res = serde_json::from_value(msg.vertexes_json);
            if res.is_err() {
                return fail!(InitEmuNet, "invalid json format for vertexes".to_string());
            }
            let input_vertexes: Vec<(u64, VertexInfo)> = res.unwrap();
            
            // deserialize the edges
            let res = serde_json::from_value(msg.edges_json);
            if res.is_err() {
                return fail!(InitEmuNet, "invalid json format for edges".to_string());
            }
            let input_edges: Vec<((u64, u64), EdgeInfo)> = res.unwrap();

            // build up the in memory graph
            let res = InMemoryGraph::from_vecs(input_vertexes, input_edges);
            if res.is_err() {
                // InMemoryGraph<u64, VertexInfo,EdgeInfo> does not implement fmt::Debug,
                // map it to () and then extract the error message
                return fail!(InitEmuNet, format!("can't build the network graph: {}", res.map(|_|{()}).unwrap_err()));
            }
            let network_graph: InMemoryGraph<u64, VertexInfo,EdgeInfo> = res.unwrap();

            // do the allocation
            let res = network_graph.partition(emunet.servers_mut());
            if res.is_err() {
                return fail!(InitEmuNet, format!("can't partition the network graph: {}", res.map(|_|{()}).unwrap_err()));
            }
            let assignment = res.unwrap();
            
            // get the lists of vertex_info and edge_info
            let (vertex_infos, edge_infos) = network_graph.into();

            // build up the list of vertexes
            let vertexes: Vec<Vertex> = vertex_infos.into_iter().map(|vi| {
                let client_id = vi.client_id;
                Vertex {
                    client_info: vi,
                    uuid: indradb::util::generate_uuid_v1(),
                    server_uuid: assignment.get(&client_id).unwrap().clone(),
                }
            }).collect();

            // prepare the list of bulk insert items
            let mut bulk_vertexes: Vec<indradb::BulkInsertItem> = Vec::new();
            let _: Vec<()> = vertexes.iter().map(|v| {
                let uuid = v.uuid.clone();
                emunet.add_vertex_assignment(v.client_info.client_id, uuid);
                let vertex = indradb::Vertex::with_id(uuid, indradb::Type::new("t").unwrap());
                bulk_vertexes.push(indradb::BulkInsertItem::Vertex(vertex));
            }).collect();
            // insert the bulk vertexes into the fucking database

            let mut bulk_vertex_properties: Vec<indradb::BulkInsertItem> = Vec::new();
            let _: Vec<()> = vertexes.into_iter().map(|v|{
                let uuid = v.uuid.clone();
                let json_value = serde_json::to_value(v).unwrap();
                bulk_vertex_properties.push(indradb::BulkInsertItem::VertexProperty(uuid, "default".to_string(), json_value));
            }).collect();
            // insert the bulk vertexes into the database
            
            succeed!(InitEmuNet, (),)
        })
    }
}