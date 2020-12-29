use std::collections::{HashMap, hash_map::ValuesMut};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::server::{ContainerServer};

#[derive(Deserialize, Serialize)]
pub struct VertexInfo {
    id: u64, // client side vertex id in the form of u64
    description: String, // a description string to hold the place
}

impl VertexInfo {
    pub fn id(&self) -> u64 {
        return self.id;
    }
}

#[derive(Deserialize, Serialize)]
pub struct EdgeInfo {
    edge_id: (u64, u64), // client side edge id in the form of (u64, u64)
    description: String, // a description string to hold the place
}

impl EdgeInfo {
    pub fn edge_id(&self) -> (u64, u64) {
        self.edge_id
    }

    pub fn description(&self) -> String {
        self.description.clone()
    }
}

#[derive(Deserialize, Serialize)]
pub struct Vertex {
    info: VertexInfo,
    uuid: uuid::Uuid,
    server_uuid: uuid::Uuid, // which server this vertex is launched on
    edges: HashMap<uuid::Uuid, Edge>
}

impl Vertex {
    pub fn new(info: VertexInfo, uuid: uuid::Uuid, server_uuid: uuid::Uuid) -> Self {
        Self{
            info, uuid, server_uuid, 
            edges: HashMap::new()
        }
    }

    pub fn id(&self) -> u64 {
        return self.info.id;
    }

    pub fn uuid(&self) -> uuid::Uuid {
        return self.uuid.clone()
    }

    pub fn add_edge(&mut self, edge: Edge) -> Result<(), String> {
        // valid the edge
        if self.uuid() != edge.edge_uuid.0 || self.edges.contains_key(&edge.edge_uuid.1) {
            return Err("invalid edge id".to_string());
        }
        if self.edges.insert(edge.edge_uuid.1.clone(), edge).is_some() {
            panic!("fatal!".to_string());
        }
        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
pub struct Edge {
    edge_uuid: (uuid::Uuid, uuid::Uuid), // out-going vertex -> incoming vertex
    description: String,
}

impl Edge {
    pub fn new(edge_uuid: (uuid::Uuid, uuid::Uuid), description: String) -> Self {
        Self{edge_uuid, description}
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum EmuNetError {
    PartitionFail(String),
    DatabaseFail(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum EmuNetState {
    Uninit,
    Working,
    Normal,
    Error(EmuNetError),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct EmuNet {
    user: String,
    name: String,
    uuid: Uuid,
    capacity: u32,
    state: EmuNetState,
    server_map: HashMap<Uuid, ContainerServer>,
    vertex_map: HashMap<u64, Uuid>,
}

impl EmuNet {
    pub fn new(user: String, name: String, uuid: Uuid, capacity: u32) -> Self {
        Self {
            user,
            name,
            uuid,
            capacity,
            state: EmuNetState::Uninit,
            server_map: HashMap::new(),
            vertex_map: HashMap::new(),
        }
    }

    pub fn add_servers(&mut self, server_list: Vec<ContainerServer>) {
        for cs in server_list.into_iter() {
            let server_id = cs.id();
            if self.server_map.contains_key(&server_id) {
                panic!("collisioned server key, this should never happen");
            }
            self.server_map.insert(server_id, cs);
        }
    }

    pub fn servers_mut<'a>(&'a mut self) ->  ValuesMut<'a, Uuid, ContainerServer>{
        self.server_map.values_mut()
    }

    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    pub fn add_vertex(&mut self, vertex_client_id: u64, vertex_uuid: Uuid) {
        self.vertex_map.insert(vertex_client_id, vertex_uuid);
    }

    pub fn vertex_type(&self) -> String {
        format!("{}.{}", &self.user, &self.name)
    }
}

impl EmuNet {
    // modifying the state of the EmuNet
    pub fn is_uninit(&self) -> bool {
        match self.state {
            EmuNetState::Uninit => true,
            _ => false,
        }
    }

    pub fn working(&mut self) {
        self.state = EmuNetState::Working;
    }

    pub fn error(&mut self, reason: EmuNetError) {
        self.state = EmuNetState::Error(reason);
    }

    pub fn normal(&mut self) {
        self.state = EmuNetState::Normal;
    }
}
